use std::path::PathBuf;
use askama::Template;
use structopt::StructOpt;

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

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "sangha")]
pub struct Args {
    #[structopt(short, long, default_value = ".")]
    pub root_dir: PathBuf,
}
