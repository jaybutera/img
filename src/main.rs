mod types;

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

fn main() -> tide::Result<()> {
    smol::block_on(main_async())
}

fn normalize_topic(topic: &str) -> String {
    topic.to_lowercase().replace(" ", "-").replace(".", "_").trim().to_string()
}

async fn main_async() -> tide::Result<()> {
    let args = types::Args::from_args();
    let port = args.port;
    log::start();

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

async fn upload_image(mut req: Request<Args>) -> tide::Result {
    // Check that content type is an image
    //if Some(ContentType::new("image/*")) == req.content_type().base {
    if let Some(mime) = req.content_type() {
        if mime.basetype() != "image"
            && mime.basetype() != "video" {
            //&& mime.essence() != "multipart/form-data" {
            // Invalid content type
            return Err(to_badreq(anyhow!("Invalid content type {}", mime.essence())));
        }

        let image = req.body_bytes().await?;
        let image_name = blake3::hash(&image).to_string();
        let image_fname = format!("{}.{}",
                image_name,
                mime.subtype());
        let topic = normalize_topic(req.param("topic")?);

        // Image path
        let mut image_path = req.state().root_dir.clone();
        image_path.push(image_fname.clone());

        // Topic path
        let mut topic_path = req.state().root_dir.clone();
        topic_path.push(format!("{}.json", topic));

        // Create topic file if not already created
        if !topic_path.exists() {
            smol::fs::write(topic_path.clone(),
                            serde_json::to_vec(&TopicData {
                                name: topic,
                                revs: vec![],
                            }).unwrap()).await?;
        }

        {
            let topic_file: AcidJson<TopicData> = AcidJson::open(&topic_path)?;
            let mut td = topic_file.write();
            td.add(vec![image_fname]);
        }

        // Write image to disk
        smol::fs::write(&image_path, image).await?;
        log::debug!("Wrote image {:?}", image_path);

        Ok("Success".into())
    } else {
        Err(to_badreq(anyhow!("No content provided")))
    }
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
