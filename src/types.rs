use askama::Template;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "upload.html")]
pub struct UploadTemplate {
    pub topic: String,
}

#[derive(Template)]
#[template(path = "topic.html")]
pub struct TopicTemplate {
    pub image_names: Vec<String>,
    pub topic: String,
}
