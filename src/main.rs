mod types;

use askama::Template;
use http_types::mime;
use smol::prelude::*;
use tide::{Request, Response};

fn main() -> tide::Result<()> {
    smol::block_on(main_async())
}

async fn main_async() -> tide::Result<()> {
    let mut app = tide::new();

    app.at("/:topic/new").get(upload_image_page);
    app.at("/:topic/new-image").post(upload_image);
    app.at("/:topic").get(get_topic_images);
    app.listen("0.0.0.0:8080").await?;

    Ok(())
}

async fn upload_image_page(mut req: Request<()>) -> tide::Result {
    let page = types::UploadTemplate {};

    let res = Response::builder(200)
        /*
        .body("<html><body><form action=\"\">
                  <input type=\"file\" id=\"myFile\" name=\"filename\">
                  <input type=\"submit\">
                </form></body></html>")
        */
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
