mod types;

use types::{TopicData, MediaUid};
use anyhow::anyhow;
use types::Args;
use std::fmt::Debug;
use std::str::FromStr;
use askama::Template;
use http_types::mime::{self, Mime};
use smol::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;
use tide::{log, Body, Request, Response, StatusCode};
use acidjson::AcidJson;

fn main() -> tide::Result<()> {
    smol::block_on(main_async())
}

fn normalize_topic(topic: &str) -> String {
    // TODO remove spaces and punctuation
    topic.to_lowercase()
}

async fn main_async() -> tide::Result<()> {
    let args = types::Args::from_args();
    let port = args.port;
    let mut app = tide::with_state(args);
    log::start();

    app.at("/").get(new_page);
    app.at("/new").get(new_page);
    app.at("/:topic/new").get(upload_image_page);
    app.at("/:topic/new-image").post(upload_image);
    //app.at("/:topic/raw").get(get_topic_images);
    app.at("/:topic").get(images_page);
    app.at("list-images/:topic").get(get_image_list);
    app.at(":topic/:name").get(get_image);
    app.listen(format!("0.0.0.0:{}", port)).await?;

    Ok(())
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
    let topic = normalize_topic(req.param("topic")?);
    let name = req.param("name")?;
    let mut path = req.state().root_dir.clone();
    //path.push(topic);
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
    path.push(topic);

    let image_list = image_list(path).await?;
    Ok(Body::from_json(&image_list)?)
}

async fn image_list(path: PathBuf) -> tide::Result<Vec<MediaUid>> {
    /*
    let image_name_stream = smol::fs::read_dir(path).await?;
    let image_names: Vec<String> = image_name_stream
        .map(|entry| entry.unwrap().file_name())
        .map(|ostr| ostr.into_string().unwrap())
        .collect().await;
    */

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
        if mime.basetype() != "image" && mime.basetype() != "video" {
            // Invalid content type
            return Err(to_badreq(anyhow!("Invalid content type {}", mime.essence())));
        }

        let image = req.body_bytes().await?;
        let image_name = blake3::hash(&image).to_string();
        let topic = normalize_topic(req.param("topic")?);
        let mut fname = req.state().root_dir.clone();
        //fname.push(topic);
        fname.push(format!("{}.{}",
                image_name,
                mime.subtype()));

        // Create topic if not already created
        // TODO Ignore result for now, failed likely means dir exists
        //smol::fs::create_dir(fname.clone()).await;

        let topic_file: AcidJson<TopicData> = AcidJson::open(&fname)?;
        let mut td = topic_file.write();
        td.add(vec![image_name]);
        //create_topic_file(topic, vec![image_name.clone()]);

        // Write image to disk
        //smol::fs::write(fname, image).await?;
        log::debug!("Wrote image {:?}", fname);

        Ok("Success".into())
    } else {
        Err(to_badreq(anyhow!("No content provided")))
    }
}

/*
async fn create_topic_file(name: String, media: Vec<MediaUid>) -> Result<(), std::io::Error> {
    let fname = format!("{}.json", name);
    let revs = vec![];
    let bytes = serde_json::to_vec(&topicdata { name, media, revs })?;

    // todo if file already exists, add a add revision?
    smol::fs::write(fname, bytes).await
}
*/

/*
async fn get_topic_images(mut req: Request<()>) -> tide::Result {
    let topic = req.param("topic")?;
    let mut image_names = smol::fs::read_dir(topic).await?;

    let name = image_names.next().await.unwrap()?;
    let image = smol::fs::read(name.path()).await?;

    let res = Response::builder(200)
        .body(image)
        .content_type(mime::JPEG)
        .build();

    Ok(res)
}
*/

fn to_badreq<E: Into<anyhow::Error> + Send + 'static + Sync + Debug>(e: E) -> tide::Error {
    tide::Error::new(StatusCode::BadRequest, e)
}

fn from_extension(extension: impl AsRef<str>) -> Option<Mime> {
    match extension.as_ref() {
        "png" => Mime::from_str("image/png").ok(),
        "jpeg" => Mime::from_str("image/jpeg").ok(),
        "mp4" => Mime::from_str("video/mp4").ok(),
        "mpeg" => Mime::from_str("video/mpeg").ok(),
        _ => Mime::from_extension(extension),
    }
}
