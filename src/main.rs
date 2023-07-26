mod types;
mod migrations;
mod utils;

use types::{Index, TopicData, MediaUid};
use anyhow::anyhow;
use types::Args;
use std::fmt::Debug;
use std::str::FromStr;
use askama::Template;
use http_types::mime::{self, Mime};
use std::path::PathBuf;
use structopt::StructOpt;
use tide::{log, Body, Request, Response, StatusCode};
use acidjson::AcidJson;
use smol::io::AsyncWriteExt;
use http_types::headers::HeaderValue;
use tide::security::{CorsMiddleware, Origin};
use async_fs::File;
use smol::io::{AsyncRead, AsyncReadExt, BufReader};
use smol::stream::StreamExt;

use crate::migrations::update_media_names;

fn main() -> tide::Result<()> {
    smol::block_on(main_async())
}

fn normalize_topic(topic: &str) -> String {
    topic.to_lowercase().replace(" ", "-").replace(".", "_").trim().to_string()
}

async fn main_async() -> tide::Result<()> {
    let args = types::Args::from_args();
    log::start();

    // If migrate is true, run migrate function instead of starting server
    if args.migrate {
        update_media_names(&args.root_dir).await?;
        return Ok(());
    }

    let port = args.port;

    let mut app = tide::with_state(args);
    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);
    app.with(cors);

    app.at("/").get(new_page);
    app.at("/new").get(new_page);
    app.at("/:topic/new").get(upload_image_page);
    app.at("/new-index").post(create_index);
    app.at("/index/:name").get(get_index);
    app.at("/:topic/new-image").post(upload_image);
    //app.at("/:topic/raw").get(get_topic_images);
    app.at("/:topic").get(images_page);
    app.at("/:topic/images").get(get_image_list);
    app.at("/img/:name").get(get_image);
    app.listen(format!("0.0.0.0:{}", port)).await?;

    Ok(())
}

async fn get_index(req: Request<Args>) -> tide::Result {
    let name = req.param("name")?;
    let mut path = req.state().root_dir.clone();
    path.push(format!("{}.json", name));

    let index = smol::fs::read_to_string(path).await?;

    let res = Response::builder(200)
        .body(index)
        .content_type(mime::JSON)
        .build();

    Ok(res)
}

/// Request contains a json file for the index which is saved into /indexes/<name>.json
async fn create_index(mut req: Request<Args>) -> tide::Result {
    let index: Index = req.body_json().await?;
    let name = normalize_topic(&index.name);
    let mut path = req.state().root_dir.clone();
    path.push(format!("{}.json", name));

    if path.exists() {
        return Err(to_badreq(anyhow!("Index already exists")));
    }

    // write index json file
    let index_str = serde_json::to_string(&index)?;
    let mut file = smol::fs::File::create(path).await?;
    file.write_all(index_str.as_bytes()).await?;

    Ok(Response::new(StatusCode::Ok))

}

async fn new_page(req: Request<Args>) -> tide::Result {
    let page = types::NewTopicTemplate {};

    let res = Response::builder(200)
        .body(page.render().unwrap())
        .content_type(mime::HTML)
        .build();

    Ok(res)
}

async fn get_image(req: Request<Args>) -> tide::Result {
    let name = req.param("name")?;
    let mut path = req.state().root_dir.clone();
    path.push(name);

    let ext = path.extension()
        .expect(&format!("Expected path {:?} to be a file but it has no extension", path))
        .to_str().unwrap();
    let mime = from_extension(ext)
        .expect(&format!("Unsupported filetype {:?} is somehow being fetched", ext));
    let image = smol::fs::read(path).await?;

    let res = Response::builder(200)
        .body(image)
        .header(tide::http::headers::ACCEPT_RANGES, "bytes")
        .content_type(mime)
        .build();

    Ok(res)
}

async fn get_image_list(req: Request<Args>) -> tide::Result<Body> {
    let topic = normalize_topic(req.param("topic")?);
    let mut path = req.state().root_dir.clone();
    path.push(format!("{}.json", topic));

    let image_list = image_list(path).await?;
    Ok(Body::from_json(&image_list)?)
}

// TODO open with AcidJson to avoid concurrency issues
async fn image_list(path: PathBuf) -> tide::Result<Vec<MediaUid>> {
    // Read the TopicData file to get the image names
    let raw_topic_data = smol::fs::read(path).await?;
    let topic_data: TopicData = serde_json::from_slice(&raw_topic_data)?;
    let image_names = topic_data.list();

    Ok(image_names)
}

async fn images_page(req: Request<Args>) -> tide::Result {
    let topic = normalize_topic(req.param("topic")?);
    let mut path = req.state().root_dir.clone();
    path.push(format!("{}.json", topic.clone()));

    let image_names = image_list(path).await?;
    let page = types::TopicTemplate {
        image_names,
        topic: topic.into()
    };

    let res = Response::builder(200)
        .body(page.render().unwrap())
        .content_type(mime::HTML)
        .build();

    Ok(res)
}

async fn upload_image_page(req: Request<Args>) -> tide::Result {
    let topic = normalize_topic(req.param("topic")?);
    let page = types::UploadTemplate {
        topic: topic.into()
    };

    let res = Response::builder(200)
        .body(page.render().unwrap())
        .content_type(mime::HTML)
        .build();

    Ok(res)
}

/// Uses the buffer to read the first 1MB of the file and hash it to get the uid
pub async fn get_uid<T>(mut reader: &mut BufReader<T>) -> anyhow::Result<(String, Vec<u8>)>
where T: AsyncRead + Unpin {
    let mut buffer = vec![0; 1024 * 1024]; // 1MB buffer

    // read exactly 1MB or the whole buffer
    let mut n = reader.read(&mut buffer).await?;
    while n < buffer.len() {
        let n2 = reader.read(&mut buffer[n..]).await?;
        if n2 == 0 {
            break;
        }
        n += n2;
    }

    // Hash the first 1MB for the uid
    let uid = blake3::hash(&buffer).to_string();

    Ok((uid, buffer))
}

/// Saves media to the filesystem and return the filename which is the hash of first 1MB of 
/// the file as the uid, and the extension of the file.
async fn save_media(mut reader: BufReader<Body>, root_dir: &PathBuf, ext: &str) -> anyhow::Result<String> {
    let (uid, mut buffer) = get_uid(&mut reader).await?;

    // Generate path
    let image_fname = format!("{}.{}", uid, ext);
    let image_path = root_dir.join(&image_fname);

    // Read and write the file
    if !image_path.exists() {
        let mut image_file = File::create(&image_path).await?;

        // Write the first chunk
        image_file.write_all(&buffer).await?;
        image_file.flush().await?;

        // Loop the rest
        while let Ok(bytes_read) = reader.read(&mut buffer).await {
            if bytes_read == 0 {
                break;
            }

            let chunk = &buffer[..bytes_read];
            image_file.write_all(chunk).await?;
            image_file.flush().await?;
        }
    }

    Ok(image_fname)
}

async fn upload_image(mut req: Request<Args>) -> tide::Result {
    let mime = req.content_type().ok_or(anyhow!("No content type"))?;

    // Invalid content type
    if mime.basetype() != "image"
            && mime.basetype() != "video" {
        return Err(to_badreq(anyhow!("Invalid content type {}", mime.essence())));
    }

    // Topic path
    let topic = normalize_topic(req.param("topic")?);
    let mut topic_path = req.state().root_dir.clone();
    topic_path.push(format!("{}.json", topic));

    // Add the image if its not already in the root dir
    let mut reader = smol::io::BufReader::new(req.take_body());
    let mut root_dir = req.state().root_dir.clone();
    let image_fname = save_media(reader, &root_dir, mime.subtype()).await?;

    // Create topic file if not already created
    if !topic_path.exists() {
        smol::fs::write(topic_path.clone(),
                        serde_json::to_vec(&TopicData {
                            name: topic,
                            revs: vec![],
                        }).unwrap()).await?;
    }

    { // Add media to topic
        let topic_file: AcidJson<TopicData> = AcidJson::open(&topic_path)?;
        let mut td = topic_file.write();
        td.add(vec![image_fname]);
    }

    Ok("Success".into())
}

fn to_badreq<E: Into<anyhow::Error> + Send + 'static + Sync + Debug>(e: E) -> tide::Error {
    tide::Error::new(StatusCode::BadRequest, e)
}

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
