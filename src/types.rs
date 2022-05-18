use std::path::PathBuf;
use askama::Template;
use structopt::StructOpt;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct HashVal([u8; 32]);

pub type MediaUid = String;

/*
impl std::fmt::Display for HashVal {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
*/

#[derive(Template)]
#[template(path = "upload.html")]
pub struct UploadTemplate {
    pub topic: String,
}

#[derive(Template)]
#[template(path = "topic.html")]
pub struct TopicTemplate {
    pub image_names: Vec<MediaUid>,
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
    // Ordered list of media files
    //pub media: Vec<MediaUid>,
    // A stack of revisions, each revision is an ordered list of specific revision operations
    /// A stack of revision operations
    pub revs: Vec<RevisionOp>,
}

impl TopicData {
    pub fn add(&mut self, media: Vec<MediaUid>) {
        self.revs.push(RevisionOp::Add(media));
    }

    pub fn rm(&mut self, media: Vec<MediaUid>) {
        self.revs.push(RevisionOp::Del(media));
    }

    pub fn list(&self) -> Vec<MediaUid> {
        let mut acc = vec![];

        for rev in self.revs.iter() {
            match rev {
                RevisionOp::Add(v) => acc.append(&mut v.clone()),
                RevisionOp::Del(v) => acc.retain(|x| !v.contains(x)),
            }
        }

        acc
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum RevisionOp {
    Add(Vec<MediaUid>),
    Del(Vec<MediaUid>),
}

/*
#[derive(Serialize, Deserialize)]
pub struct TopicRevisionHistory {
    /// Topic name
    pub topic: String,
    // A stack of revisions, each revision is an ordered list of specific revision operations
    pub revs: Vec<Vec<RevisionOp>>,
}
*/
