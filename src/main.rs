mod types;

use askama::Template;
use http_types::mime;
use smol::prelude::*;
use tide::{Body, Request, Response};

fn main() -> tide::Result<()> {
    smol::block_on(main_async())
}

async fn main_async() -> tide::Result<()> {
    let mut app = tide::new();

    app.at("/:topic/new").get(upload_image_page);
    app.at("/:topic/new-image").post(upload_image);
    app.at("/:topic/raw").get(get_topic_images);
    app.at("/:topic").get(images_page);
    app.at("list-images/:topic").get(get_image_list);
    app.at(":topic/:name").get(get_image);
    app.listen("0.0.0.0:8080").await?;

    Ok(())
}

async fn get_image(mut req: Request<()>) -> tide::Result {
    let topic = req.param("topic")?;
    let name = req.param("name")?;
    let path = format!("{}/{}", topic, name);
    let image = smol::fs::read(path).await?;

    let res = Response::builder(200)
        .body(image)
        .content_type(mime::JPEG)
        .build();

    Ok(res)
}

async fn get_image_list(mut req: Request<()>) -> tide::Result<Body> {
    let topic = req.param("topic")?;
    let image_list = image_list(topic.into()).await?;
    Ok(Body::from_json(&image_list)?)
}

async fn image_list(topic: String) -> tide::Result<Vec<String>> {
    let image_name_stream = smol::fs::read_dir(topic).await?;
    let image_names: Vec<String> = image_name_stream
        .map(|entry| entry.unwrap().file_name())
        .map(|ostr| ostr.into_string().unwrap())
        .collect().await;

    Ok(image_names)
}

async fn images_page(mut req: Request<()>) -> tide::Result {
    let topic = req.param("topic")?;
    let image_names = image_list(topic.into()).await?;
    let page = types::TopicTemplate { image_names };

    let res = Response::builder(200)
        .body(page.render().unwrap())
        .content_type(mime::HTML)
        .build();


    Ok(res)
}

async fn upload_image_page(mut req: Request<()>) -> tide::Result {
    let page = types::UploadTemplate {};

    let res = Response::builder(200)
        .body(page.render().unwrap())
        .content_type(mime::HTML)
        .build();


    Ok(res)
}

async fn upload_image(mut req: Request<()>) -> tide::Result {
    // Check that content type is an image
    //if Some(ContentType::new("image/*")) == req.content_type().base {
    //} else {
    //}
    let image = req.body_bytes().await?;

    // Write image to disk
    let topic = req.param("topic")?;
    let fname = format!("./{}/{}.jpeg", topic, blake3::hash(&image));
    println!("Wrote image {}", fname);

    smol::fs::write(fname, image).await?;

    Ok("Success".into())
}

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
