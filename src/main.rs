use tide::Request;

fn main() -> tide::Result<()> {
    smol::block_on(main_async())
}

async fn main_async() -> tide::Result<()> {
    let mut app = tide::new();

    app.at("/:topic/new").post(upload_image);
    app.listen("0.0.0.0:8080").await?;

    Ok(())
}

async fn upload_image(mut req: Request<()>) -> tide::Result {
    // Check that content type is an image
    //if Some(ContentType::new("image/*")) == req.content_type().base {
    //} else {
    //}
    let image = req.body_bytes().await?;

    // Write image to disk
    let fname = format!("{}.jpeg", blake3::hash(&image));
    smol::fs::write(fname, image).await?;

    Ok("Heyo".into())
}
