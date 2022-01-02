use std::path::PathBuf;
use askama::Template;
use structopt::StructOpt;
use serde::{Serialize, Deserialize};

pub type MediaUid = String;

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

#[derive(Template)]
#[template(path = "new.html")]
pub struct NewTopicTemplate {}

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "sangha")]
pub struct Args {
    #[structopt(short, long, default_value = ".")]
    pub root_dir: PathBuf,
    #[structopt(short, long, default_value = "2342")]
    pub port: u32,
}

#[derive(Serialize, Deserialize)]
pub struct TopicData {
    /// Topic name
    pub name: String,
    /// Ordered list of media files
    pub media: Vec<MediaUid>,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum RevisionOp {
    Add(Vec<MediaUid>),
    Del(Vec<MediaUid>),
}

#[derive(Serialize, Deserialize)]
pub struct TopicRevisionHistory {
    /// Topic name
    pub topic: String,
    // A stack of revisions, each revision is an ordered list of specific revision operations
    pub revs: Vec<Vec<RevisionOp>>,
}
