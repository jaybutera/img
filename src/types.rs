use askama::Template;
use serde::Serialize;

#[derive(Template)]
#[template(path = "upload.html")]
pub struct UploadTemplate {}

#[derive(Template)]
#[template(path = "topic.html")]
pub struct TopicTemplate {}

#[derive(Serialize)]
pub struct ImageList(pub Vec<String>);
