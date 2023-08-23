mod types;
mod migrations;
mod utils;

use actix_session::Session;
use actix_web::{web, App, HttpServer, HttpResponse, Result, HttpRequest, post, get};
use actix_cors::Cors;
use types::{
    AnyError,
    VerificationPayload,
    ServerState,
    Args,
    TopicData,
    MediaUid};
use ed25519_dalek::{SigningKey, Signature, Verifier, VerifyingKey};
use anyhow::anyhow;
use std::str::FromStr;
use http_types::mime::{self, Mime};
use std::path::PathBuf;
use acidjson::AcidJson;
use rand_core;
use smol::stream::StreamExt;
use structopt::StructOpt;

use crate::utils::{
    save_file,
    //save_media,
    save_thumbnail,
    get_index_paths,
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

/*
async fn main_async() -> Result<()> {
    let args = types::Args::from_args();
    log::start();

    // If migrate is true, run migrate function instead of starting server
    if args.migrate {
        generate_thumbnails(&args.root_dir).await?;
        //update_media_names(&args.root_dir).await?;
        return Ok(());
    }

    let port = args.port;

    // Concurrently maintain a queue of thumbnails to generate,
    // at most N at a time
    let mut thumbnail_queue = async_channel::unbounded::<PathBuf>();
    let mut thumbnail_queue_sender = thumbnail_queue.0.clone();
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


    let state = ServerState {
        args: args.clone(),
        thumbnail_sender: thumbnail_queue_sender,
    };

    let mut app = with_state(state);
    use http::cookies::SameSite;
    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        //.allow_origin(Origin::from("*"))
        .allow_origin(Origin::from("http://localhost:5173"))
        .allow_credentials(true);
    let sessions = sessions::SessionMiddleware::new(
        sessions::MemoryStore::new(),
        &"sessionasdfsdfsdfsdfsdfsdfsdfsdfsdfsdfsdf".to_string().into_bytes(),
        //args.session_key.as_bytes(),
    )
    .with_same_site_policy(SameSite::Lax);
    app.with(sessions);
    app.with(cors);

    app.at("/tag/:name").get(get_index);
    //app.at("/all-indexes").get(get_index_list);
    app.at("/{topic}/new-image").post(upload_image);
    app.at("/{topic}/images").get(get_image_list);
    app.at("/{topic}/tags").get(get_tag_list);
    app.at("/{topic}/new-tag").post(add_tag_to_topic);
    app.at("/{topic}/remove-tag").post(rm_tag_from_topic);
    app.at("/thumbnail/:name").get(get_image_thumbnail);
    app.at("/img/:name").get(get_image_full);
    app.at("/generate-key").get(generate_keys);
    app.at("/generate-challenge").get(generate_challenge);
    app.at("/authenticate").post(authenticate);
    app.listen(format!("0.0.0.0:{}", port)).await?;

    Ok(())
}
*/

#[get("/generate-key")]
async fn generate_keys() -> Result<HttpResponse> {
    let keypair = SigningKey::generate(&mut rand_core::OsRng);
    // Hash with sha512 for 64 bytes
    //let hash = sha3::Sha3_512::digest(&keypair.to_bytes());
    let encoded = base64::encode(&keypair.to_bytes());

    Ok(HttpResponse::Ok().body(encoded))
}

#[get("/generate-challenge")]
async fn generate_challenge(
    session: Session,
) -> Result<HttpResponse> {
    let challenge: [u8; 32] = rand::random();

    // Store the challenge in the session
    session.insert("challenge", challenge.to_vec())?;

    // Get the sid from the session
    //let sid = session.id();
    //info!("Session ID: {}", sid);

    let challenge = base64::encode(challenge);

    Ok(HttpResponse::Ok().body(challenge))
}

#[post("/authenticate")]
async fn authenticate(
    session: Session,
    //data: web::Data<ServerState>,
    payload: web::Json<VerificationPayload>,
) -> Result<HttpResponse> {
    let pubkey: [u8; 32] = payload.public_key[..].try_into()
        .map_err(|_| AnyError::from(anyhow!("Invalid public key length!")))?;
    let public_key = VerifyingKey::from_bytes(&pubkey)
        .map_err(|_| AnyError::from(anyhow!("Invalid public key!")))?;

    //let sid = req.session().id();
    //info!("Session ID: {}", sid);

    // Check if the challenge in the session matches the provided challenge
    let stored_challenge = session.get::<Vec<u8>>("challenge")?
        .ok_or(AnyError::from(anyhow!("No challenge found in session!")))?;

    // Remove the challenge from the session to ensure it can't be used again
    session.remove("challenge");

    let sig: [u8; 64] = payload.signature[..].try_into()
        .map_err(|_| AnyError::from(anyhow!("Invalid signature length!")))?;
    let signature = Signature::from_bytes(&sig);
    if public_key.verify(&stored_challenge, &signature).is_ok() {
        session.insert("verified", true)?;
        Ok(HttpResponse::Ok().finish())
    } else {
        Err(AnyError::from(anyhow!("Signature is invalid!")).into())
    }
}

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
    //let sid = req.session().id();
    //info!("Session ID: {}", sid);
    //let name = req.param("name")?;
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

async fn get_image(path: &PathBuf) -> Result<(Vec<u8>, mime::Mime), std::io::Error> {
    let ext = path.extension()
        .expect(&format!("Expected path {:?} to be a file but it has no extension", path))
        .to_str().unwrap();
    let mime = from_extension(ext)
        .expect(&format!("Unsupported filetype {:?} is somehow being fetched", ext));
    let image = smol::fs::read(path).await?;
    Ok((image, mime))
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

// TODO open with AcidJson to avoid concurrency issues
async fn image_list(path: PathBuf) -> Result<Vec<MediaUid>> {
    // Read the TopicData file to get the image names
    let raw_topic_data = smol::fs::read(path).await?;
    let topic_data: TopicData = serde_json::from_slice(&raw_topic_data)?;
    let image_names = topic_data.list();

    Ok(image_names)
}

/*
#[post("{topic}/new-image")]
async fn upload_image(
    webpath: web::Path<String>,
    session: Session,
    mut req: HttpRequest,
    payload: web::Json<Vec<u8>>,
    data: web::Data<ServerState>,
) -> Result<HttpResponse> {
    let topic = normalize_topic(&webpath.into_inner());
    //let mime = req.content_type().ok_or(anyhow!("No content type"))?;
    use actix_web::http::header;
    let mime = req.headers().get(header::CONTENT_TYPE)
        .ok_or(AnyError::from(anyhow!("No content type")))?
        .to_str().unwrap().parse::<mime::Mime>()
        .map_err(|e| AnyError::from(e.into()))?;

    // Invalid content type
    if mime.basetype() != "image"
            && mime.basetype() != "video" {
        return Err(AnyError::from(anyhow!("Invalid content type {}", mime.essence())).into());
    }

    // Topic path
    let mut topic_path = data.args.root_dir.clone();
    topic_path.push(format!("{}.json", topic));

    // Verify user
    if session.get("verified") != Ok(Some(true)) {
        return Err(AnyError::from(anyhow!("Not verified please authenticate.")).into());
    }

    // Add the image if its not already in the root dir
    //let reader = smol::io::BufReader::new(req.take_body());
    let reader = smol::io::BufReader::new(payload.into_inner());
    let root_dir = data.args.root_dir.clone();
    let image_fname = save_media(
        reader,
        &root_dir,
        mime.subtype(),
        data.thumbnail_sender.clone()).await
            .map_err(|e| AnyError::from(e.into()))?;

    // Create topic file if not already created
    if !topic_path.exists() {
        smol::fs::write(topic_path.clone(),
                        serde_json::to_vec(&TopicData {
                            name: topic,
                            revs: vec![],
                        }).unwrap()).await?;
    }

    { // Add media to topic
        let topic_file: AcidJson<TopicData> = AcidJson::open(&topic_path)
            .map_err(|e| AnyError::from(e.into()))?;
        let mut td = topic_file.write();
        td.add(vec![image_fname]);
    }

    Ok(HttpResponse::Ok().finish())
}
*/

#[post("{topic}/new-image")]
async fn upload_image(
    req: HttpRequest,
    //mut payload: Multipart,
    webpath: web::Path<String>,
    //bytes: web::Bytes,
    payload: web::Payload,
    data: web::Data<ServerState>,
    session: Session,
) -> Result<HttpResponse> {
    let topic = normalize_topic(&webpath.into_inner());
    let root_dir = data.args.root_dir.clone();
    // Get Content-Type header
    let mime = req
        .headers()
        .get("Content-Type")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("No content type"))?
        .to_str()
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid content type"))?;
    let ext = mime::Mime::from_str(mime)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid content type"))?;
    let ext = ext.subtype();

    // Invalid content type
    if !mime.starts_with("image/") && !mime.starts_with("video/") {
        return Err(actix_web::error::ErrorBadRequest(format!(
            "Invalid content type {}",
            mime
        )));
    }

    // Topic path
    let mut topic_path = data.args.root_dir.clone();
    topic_path.push(format!("{}.json", topic));

    // Verify user
    session.get("verified")?
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Not verified please authenticate"))?;

    // Add the image if its not already in the root dir
    let image_fname = save_file(
        &root_dir,
        payload,
        ext,
        data.thumbnail_sender.clone()).await
            .map_err(|e| AnyError::from(e))?;

    // Create topic file if not already created
    if !topic_path.exists() {
        smol::fs::write(topic_path.clone(),
                        serde_json::to_vec(&TopicData {
                            name: topic,
                            revs: vec![],
                        }).unwrap()).await?;
    }

    { // Add media to topic
        let topic_file: AcidJson<TopicData> = AcidJson::open(&topic_path)
            .map_err(|e| AnyError::from(anyhow!("{}", e)))?;
        let mut td = topic_file.write();
        td.add(vec![image_fname]);
    }

    Ok(HttpResponse::Ok().body("Success"))
}


/*
fn to_badreq<E: Into<anyhow::Error> + Send + 'static + Sync + Debug>(e: E) -> Error {
    Error::new(StatusCode::BadRequest, e)
}
*/

fn from_extension(extension: impl AsRef<str>) -> Option<Mime> {
    match extension.as_ref() {
        "png" => Mime::from_str("image/png").ok(),
        "jpeg" => Mime::from_str("image/jpeg").ok(),
        "jpg" => Mime::from_str("image/jpeg").ok(),
        "mp4" => Mime::from_str("video/mp4").ok(),
        "mpeg" => Mime::from_str("video/mpeg").ok(),
        _ => Mime::from_extension(extension),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let args = Args::from_args();
    let port = args.port;
    //log::start();

    // If migrate is true, run migrate function instead of starting server
    if args.migrate {
        //generate_thumbnails(&args.root_dir).await?;
        //update_media_names(&args.root_dir).await?;
        return Ok(());
    }

    let thumbnail_sender = thumbnail_generator(&args).await;
    let state = ServerState {
        args: args.clone(),
        thumbnail_sender,
    };

    use actix_web::web::Data;
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            //.wrap(middleware::Compress::default())
            .wrap(Cors::permissive())
            .service(get_index)
            .service(upload_image)
            .service(get_image_list)
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
