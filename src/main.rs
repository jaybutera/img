mod types;

use types::Args;
use askama::Template;
use http_types::mime;
use smol::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;
use tide::{log, Body, Request, Response};

fn main() -> tide::Result<()> {
    smol::block_on(main_async())
}

async fn main_async() -> tide::Result<()> {
    let args = types::Args::from_args();
    let mut app = tide::with_state(args);
    log::start();

    app.at("/:topic/new").get(upload_image_page);
    app.at("/:topic/new-image").post(upload_image);
    //app.at("/:topic/raw").get(get_topic_images);
    app.at("/:topic").get(images_page);
    app.at("list-images/:topic").get(get_image_list);
    app.at(":topic/:name").get(get_image);
    app.listen("0.0.0.0:8080").await?;

    Ok(())
}

async fn get_image(req: Request<Args>) -> tide::Result {
    let topic = req.param("topic")?;
    let name = req.param("name")?;
    let mut path = req.state().root_dir.clone();
    path.push(topic);
    path.push(name);

    let image = smol::fs::read(path).await?;

    let res = Response::builder(200)
        .body(image)
        .content_type(mime::JPEG)
        .build();

    Ok(res)
}

async fn get_image_list(req: Request<Args>) -> tide::Result<Body> {
    let topic = req.param("topic")?;
    let mut path = req.state().root_dir.clone();
    path.push(topic);

    let image_list = image_list(path).await?;
    Ok(Body::from_json(&image_list)?)
}

async fn image_list(path: PathBuf) -> tide::Result<Vec<String>> {
    let image_name_stream = smol::fs::read_dir(path).await?;
    let image_names: Vec<String> = image_name_stream
        .map(|entry| entry.unwrap().file_name())
        .map(|ostr| ostr.into_string().unwrap())
        .collect().await;

    Ok(image_names)
}

async fn images_page(req: Request<Args>) -> tide::Result {
    let topic = req.param("topic")?;
    let mut path = req.state().root_dir.clone();
    path.push(topic);

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
    let topic = req.param("topic")?;
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
    //} else {
    //}
    let image = req.body_bytes().await?;
    let topic = req.param("topic")?;
    let mut fname = req.state().root_dir.clone();//.to_str().unwrap();
    fname.push(topic);

    // Create topic if not already created
    // TODO Ignore result for now, failed likely means dir exists
    smol::fs::create_dir(fname.clone()).await;

    // Write image to disk
    fname.push(format!("{}.jpeg", blake3::hash(&image)));
    log::debug!("Wrote image {:?}", fname);

    smol::fs::write(fname, image).await?;

    Ok("Success".into())
}

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
