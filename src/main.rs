mod types;
mod migrations;
mod utils;

use actix_session::Session;
use actix_web::{web, App, HttpServer, HttpResponse, Result, HttpRequest, post, get};
use actix_cors::Cors;
use types::{
    PublicKey,
    AnyError,
    VerificationPayload,
    ServerState,
    Args,
    TopicData,
    MediaUid};
use ed25519_dalek::{SigningKey, Signature, Verifier, VerifyingKey};
use anyhow::anyhow;
use std::path::PathBuf;
use acidjson::AcidJson;
use mime::Mime;
use rand_core;
use smol::stream::StreamExt;
use structopt::StructOpt;
use actix_multipart::{form::tempfile::TempFile, Field, Multipart};
use types::{
    mimes::from_ext,
    ServerErr,
};

use crate::utils::{
    mime_and_ext,
    get_image,
    ext,
    get_topic_owner,
    is_valid_media,
    save_file,
    save_thumbnail,
    get_tags_for_topic,
    add_tag_for_topic,
    rm_tag_for_topic,
};

fn normalize_topic(topic: &str) -> String {
    topic.to_lowercase()
        .replace(" ", "-")
        .replace("%20", "-")
        .replace(".", "_")
        .trim().to_string()
}

/// Start the thumbnail generator process which generates for all files passed on the channel
async fn thumbnail_generator(args: &Args) -> smol::channel::Sender<PathBuf> {
    // Concurrently maintain a queue of thumbnails to generate,
    // at most N at a time
    let thumbnail_queue = async_channel::unbounded::<PathBuf>();
    let thumbnail_queue_sender = thumbnail_queue.0.clone();
    let mut thumbnail_queue_receiver = thumbnail_queue.1.clone();

    let mut thumbnail_path = args.root_dir.clone();
    thumbnail_path.push("thumbnails");

    smol::spawn(async move {
        while let Some(path) = thumbnail_queue_receiver.next().await {
            let max_thumbnail_size = 500;
            let res = save_thumbnail(
                path.clone(), thumbnail_path.clone(),
                max_thumbnail_size)
                .await;
            if let Err(e) = res {
                log::error!("Error saving thumbnail: {}", e);
            }
        }
    }).detach();

    thumbnail_queue_sender
}

#[get("/generate-key")]
async fn generate_keys() -> Result<HttpResponse> {
    let secret = SigningKey::generate(&mut rand_core::OsRng);
    let encoded = base64::encode(&secret.to_bytes());

    Ok(HttpResponse::Ok().body(encoded))
}

#[get("/generate-challenge")]
async fn generate_challenge(
    session: Session,
) -> Result<HttpResponse> {
    let challenge: [u8; 32] = rand::random();
    // Store the challenge in the session
    session.insert("challenge", challenge.to_vec())?;

    let challenge = base64::encode(challenge);
    Ok(HttpResponse::Ok().json(challenge))
}

#[post("/authenticate")]
async fn authenticate(
    session: Session,
    payload: web::Json<VerificationPayload>,
) -> Result<HttpResponse> {
    // Check if the challenge in the session matches the provided challenge
    let stored_challenge = session.get::<Vec<u8>>("challenge")?
        .ok_or(AnyError::from(anyhow!("No challenge found in session!")))?;
    // Remove the challenge from the session to ensure it can't be used again
    session.remove("challenge");

    //let pubkey: PublicKey = payload.public_key[..].try_into()
        //.map_err(|_| AnyError::from(anyhow!("Invalid public key length!")))?;
    let public_key = VerifyingKey::from_bytes(&payload.public_key.to_bytes())
        .map_err(|_| AnyError::from(anyhow!("Invalid public key!")))?;

    let sig: [u8; 64] = payload.signature[..].try_into()
        .map_err(|_| AnyError::from(anyhow!("Invalid signature length!")))?;
    let signature = Signature::from_bytes(&sig);

    if public_key.verify(&stored_challenge, &signature).is_ok() {
        session.insert("verified_pubkey", payload.public_key.to_string())?;
        Ok(HttpResponse::Ok().finish())
    } else {
        Err(AnyError::from(anyhow!("Signature is invalid!")).into())
    }
}

/*
async fn get_index_list(
    data: web::Data<ServerState>,
) -> Result<HttpResponse> {
    let path = data.args.root_dir.clone();
    let paths: Vec<PathBuf> = get_index_paths(&path).await
        .map_err(|e| AnyError::from(e))?;

    // map to just the names
    let paths: Vec<String> = paths.iter()
        .map(|p| p.file_stem()
            .expect("index does not have a file stem").to_str()
            .expect("can't convert index filestem to string").to_string())
        .collect();

    Ok(HttpResponse::Ok().json(paths))
}
*/

#[get("/tag/{name}")]
async fn get_index(
    webpath: web::Path<String>,
    data: web::Data<ServerState>,
) -> Result<HttpResponse> {
    let name = normalize_topic(&webpath.into_inner());
    let mut path = data.args.root_dir.clone();
    path.push("indexes");
    path.push(format!("{}.json", name));

    let index = smol::fs::read_to_string(path).await?;

    Ok(HttpResponse::Ok().body(index))
}

#[get("/img/{name}")]
async fn get_image_full(
    webpath: web::Path<String>,
    data: web::Data<ServerState>
) -> Result<HttpResponse> {
    let name = webpath.into_inner();
    let mut path = data.args.root_dir.clone();
    path.push(name);
    let (image, mime) = get_image(&path).await?;

    Ok(HttpResponse::Ok()
        .header("Accept-Ranges", "bytes")
        .content_type(mime.to_string())
        .body(image))
}

#[get("/thumbnail/{name}")]
async fn get_image_thumbnail(
    webpath: web::Path<String>,
    data: web::Data<ServerState>
) -> Result<HttpResponse> {
    let name = webpath.into_inner();
    let mut path = data.args.root_dir.clone();
    // Use the thumbnail
    path.push("thumbnails");
    path.push(name);
    let (image, mime) = get_image(&path).await?;

    Ok(HttpResponse::Ok()
        .header("Accept-Ranges", "bytes")
        .content_type(mime.to_string())
        .body(image))
}

#[post("rm-tag/{topic}/{tag}")]
async fn rm_tag_from_topic(
    webpath: web::Path<(String, String)>,
    data: web::Data<ServerState>,
) -> Result<HttpResponse> {
    let (topic, tag) = webpath.into_inner();
    let topic = normalize_topic(&topic);
    rm_tag_for_topic(&data.args.root_dir, topic, tag).await
        .map_err(|e| AnyError::from(e))?;

    Ok(HttpResponse::Ok().finish())
}

#[post("new-tag/{topic}/{tag}")]
async fn add_tag_to_topic(
    webpath: web::Path<(String, String)>,
    data: web::Data<ServerState>,
) -> Result<HttpResponse> {
    let (topic, tag) = webpath.into_inner();
    let topic = normalize_topic(&topic);
    add_tag_for_topic(&data.args.root_dir, topic, tag).await
        .map_err(|e| AnyError::from(e))?;

    Ok(HttpResponse::Ok().finish())
}

#[get("{topic}/tags")]
async fn get_tag_list(
    webpath: web::Path<String>,
    data: web::Data<ServerState>,
) -> Result<HttpResponse> {
    let topic = normalize_topic(&webpath.into_inner());
    let path = data.args.root_dir.clone();

    let tags = get_tags_for_topic(&path, &topic).await
        .map_err(|e| AnyError::from(e))?;

    Ok(HttpResponse::Ok().json(tags))
}

//#[cfg(feature = "multiplayer")]
#[get("{id}/{topic}/images")]
async fn get_image_list_by_id(
    webpath: web::Path<(String, String)>,
    data: web::Data<ServerState>,
    session: Session,
) -> Result<HttpResponse> {
    let (id, topic) = &webpath.into_inner();
    let topic = normalize_topic(topic);

    let mut path = data.args.root_dir.clone();
    path.push(format!("{}.json", topic));

    let owner = get_topic_owner(&path)
        .map_err(|e| AnyError::from(e))?;
    is_verified(&id, &owner.to_string(), &session)?;

    let image_list = image_list(path).await?;
    Ok(HttpResponse::Ok().json(image_list))
}

/*
#[cfg(not(feature = "multiplayer"))]
#[get("{topic}/images")]
async fn get_image_list(
    webpath: web::Path<String>,
    data: web::Data<ServerState>,
) -> Result<HttpResponse> {
    let topic = normalize_topic(&webpath.into_inner());
    let mut path = data.args.root_dir.clone();
    path.push(format!("{}.json", topic));

    let image_list = image_list(path).await?;
    Ok(HttpResponse::Ok().json(image_list))
}
*/

// TODO open with AcidJson to avoid concurrency issues
async fn image_list(path: PathBuf) -> Result<Vec<MediaUid>> {
    // Read the TopicData file to get the image names
    let raw_topic_data = smol::fs::read(path).await?;
    let topic_data: TopicData = serde_json::from_slice(&raw_topic_data)?;
    let image_names = topic_data.list();

    Ok(image_names)
}

//#[cfg(feature = "multiplayer")]
#[post("{id}/{topic}/new-image")]
async fn upload_image_by_id(
    req: HttpRequest,
    webpath: web::Path<(String, String)>,
    //payload: web::Payload,
    mut payload: Multipart,
    data: web::Data<ServerState>,
    session: Session,
) -> Result<HttpResponse> {
    let (id, topic) = &webpath.into_inner();
    let topic = normalize_topic(topic);
    let root_dir = data.args.root_dir.clone();

    while let Some(mut field) = payload.try_next().await? {
    let (mime, ext) = mime_and_ext(&field)?;
    is_valid_media(&mime)?;

    // Topic path
    let mut topic_path = data.args.root_dir.clone();
    topic_path.push(format!("{}.json", topic));

    // If topic exists, verify the owner
    let owner = get_topic_owner(&topic_path)
        .map_err(|e| AnyError::from(e))?;
    is_verified(&id, &owner.to_string(), &session)?;
    log::debug!("Verified owner");

    // Add the image if its not already in the root dir
    let image_fname = save_file(
        &root_dir,
        //payload,
        field,
        ext,
        data.thumbnail_sender.clone()).await?;

    /*
    { // Add media to topic
        let topic_file: AcidJson<TopicData> = AcidJson::open(&topic_path)
            .map_err(|e| AnyError::from(anyhow!("{}", e)))?;
        let mut td = topic_file.write();
        td.add(vec![image_fname]);
    }
    */
    // Add media to topic db
    let td = if let Some(bytes) = data.topic_db.get(&topic)
        .map_err(|e| ServerErr::from(e))?
    {
        let mut td: TopicData = serde_json::from_slice(bytes.as_ref())?;
        td.add(vec![image_fname]);
        td
    } else {
        TopicData::new(topic.clone(), None, vec![image_fname])
    };
    let bytes = serde_json::to_vec(&td)?;
    // TODO key should be topic and owner id together
    data.topic_db.insert(&topic, bytes)
        .map_err(|e| ServerErr::from(e))?;
    }

    Ok(HttpResponse::Ok().body("Success"))
}

/*
#[derive(Debug, MultipartForm)]
struct Upload {
    files: Vec<TempFile>,
}
*/

/*
#[cfg(not(feature = "multiplayer"))]
#[post("{topic}/new-image")]
async fn upload_image(
    //req: HttpRequest,
    webpath: web::Path<String>,
    payload: web::Payload,
    //mut payload: Multipart,
    //mut payload: MultipartForm<Upload>,
    data: web::Data<ServerState>,
) -> Result<HttpResponse> {
    let topic = normalize_topic(&webpath.into_inner());
    let root_dir = data.args.root_dir.clone();

    // Topic path
    let mut topic_path = data.args.root_dir.clone();
    topic_path.push(format!("{}.json", topic));

    log::debug!("Topic path: {:?}", topic_path);

    //let mut writer_tasks = vec![];
    //while let Some(field) = payload.try_next().await? {
        log::debug!("Got field");
        let (mime, ext) = mime_and_ext(&field)?;
        log::debug!("Mime: {:?}, ext: {:?}", mime, ext);
        is_valid_media(&mime)?;

        // Add the image and fail if it already exists
        let task = save_file(
            &root_dir,
            field,
            ext,
            data.thumbnail_sender.clone());

        writer_tasks.push(task);
        log::debug!("Added task");
    //}

    // Wait for all files to save
    log::debug!("Waiting for all files to save");
    /*
    let mut image_fnames = vec![];
    for task in writer_tasks {
        let fname = task.await?;
        image_fnames.push(fname);
    }
    log::debug!("Fnames: {:?}", image_fnames.clone());

    // Write to topic
    let td = if let Some(bytes) = data.topic_db.get(&topic)
        .map_err(|e| ServerErr::from(e))?
    {
        let mut td: TopicData = serde_json::from_slice(bytes.as_ref())?;
        td.add(image_fnames);
        td
    } else {
        TopicData::new(topic.clone(), None, image_fnames)
    };

    let bytes = serde_json::to_vec(&td)?;
    data.topic_db.insert(&topic, bytes)
        .map_err(|e| ServerErr::from(e))?;
    */

    Ok(HttpResponse::Ok().body("Success"))
}
*/

/*
async fn multipart_guard(req: HttpRequest, srv: &actix_web::web::ServiceRequest) -> Result<actix_service::Either<HttpResponse, HttpRequest>, Error> {
    if let Some(content_type) = req.headers().get("content-type") {
        if let Ok(content_type_str) = content_type.to_str() {
            if content_type_str.starts_with("multipart/form-data") {
                return Ok(actix_service::Either::B(req));
            }
        }
    }
    Ok(actix_service::Either::A(req.error_response(HttpResponse::BadRequest().body("Invalid content type"))))
}
*/

/// Check that the id, owner and session public key all match
fn is_verified(
    id: &str,
    owner: &str,
    session: &Session,
) -> Result<()> {
    if id.len() != 8 || owner.len() < 8 {
        return Err(actix_web::error::ErrorInternalServerError("Invalid id, owner, or pubkey length"));
    }

    // Owner should match id
    if owner[..8] != id[..] {
        return Err(actix_web::error::ErrorInternalServerError("Verified public key does not match provided id"));
    }

    // Session key should match id
    let pubkey: String = session.get("verified_pubkey")?
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Not verified please authenticate"))?;

    if pubkey.len() >= 8 && pubkey[..8] != id[..] {
        return Err(actix_web::error::ErrorInternalServerError("Verified public key does not match provided id"));
    }

    Ok(())
}


/*
fn to_badreq<E: Into<anyhow::Error> + Send + 'static + Sync + Debug>(e: E) -> Error {
    Error::new(StatusCode::BadRequest, e)
}
*/

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new(
            ).default_filter_or("trace actix_web=trace, img=trace"));
    /*
    Builder::from_env(env::var("RUST_LOG").unwrap_or_else(|_| "debug".to_string()))
        .format(|buf, record| {
            writeln!(buf,
                "{} [{}] - {}:{}: {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.file().unwrap_or("<unknown>"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .filter(None, LevelFilter::Debug)
        .init();
    */

    let args = Args::from_args();
    let port = args.port;

    // If migrate is true, run migrate function instead of starting server
    if args.migrate {
        //generate_thumbnails(&args.root_dir).await?;
        //update_media_names(&args.root_dir).await?;
        return Ok(());
    }

    let db = sled::open(&args.db_path).unwrap();
    let tree = db.open_tree("topic_db").unwrap();

    let thumbnail_sender = thumbnail_generator(&args).await;
    let state = ServerState {
        args: args.clone(),
        topic_db: tree,
        thumbnail_sender,
    };

    use actix_web::web::Data;
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .wrap(actix_web::middleware::Compress::default())
            .wrap(Cors::permissive())
            .service(get_index)
            .service(upload_image_by_id)
            .service(get_image_list_by_id)
            .service(get_tag_list)
            .service(add_tag_to_topic)
            .service(rm_tag_from_topic)
            .service(get_image_thumbnail)
            .service(get_image_full)
            .service(generate_keys)
            .service(generate_challenge)
            .service(authenticate)
            .wrap(actix_web::middleware::Logger::default())
            // TODO use a better session key and secure it
            .wrap(actix_session::CookieSession::signed(&[0; 32]).secure(false))
    })
    .bind(format!("localhost:{}", port))?
    .run()
    .await
}
