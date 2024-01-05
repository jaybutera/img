pub mod mimes;
pub mod topic;
pub mod crypto;

use std::path::PathBuf;
use structopt::StructOpt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashSet;
use thiserror::Error;
//use acidjson::AcidJson;
use anyhow::anyhow;
use log::info;
use crate::PublicKey;

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct HashVal([u8; 32]);

// TODO AnyError is now legacy, remove it
#[derive(Debug)]
pub struct AnyError {
    err: anyhow::Error,
}

impl actix_web::ResponseError for AnyError {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::InternalServerError().body(self.err.to_string())
    }
}

impl From<anyhow::Error> for AnyError {
    fn from(err: anyhow::Error) -> Self {
        Self { err }
    }
}

impl::std::fmt::Display for AnyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.err)
    }
}

/*
impl AnyError {
    pub fn anyhow(s: &str) -> Self {
        AnyError::from(anyhow::anyhow!(s))
    }
}
*/

#[derive(Error, Debug)]
pub enum ServerErr {
    #[error("Error with topic database")]
    TopicDbError(#[from] sled::Error),
    #[error("Error topic not found: `{0}`")]
    TopicNotFound(String),
    #[error("Filetype Error: `{0}`")]
    FiletypeError(String),
    #[error("IO Error: `{0}`")]
    IOError(#[from] std::io::Error),
    #[error("Error: `{0}`")]
    InvalidExtension(String),
    #[error("Error: `{0}`")]
    CustomError(#[from] anyhow::Error),
}

impl actix_web::ResponseError for ServerErr {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::InternalServerError().body(self.to_string())
    }
}


#[derive(Serialize, Deserialize)]
pub struct VerificationPayload {
    pub public_key: PublicKey,
    pub signature: Vec<u8>,
}

#[derive(Clone)]
pub struct ServerState {
    pub args: Args,
    pub topic_db: sled::Tree,
    pub thumbnail_sender: smol::channel::Sender<PathBuf>,
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "sangha")]
pub struct Args {
    #[structopt(short, long, default_value = ".")]
    pub root_dir: PathBuf,
    #[structopt(short, long, default_value = "2342")]
    pub port: u32,
    #[structopt(short, long)]
    pub migrate: bool,
    #[structopt(short, long, default_value = "./topic_db")]
    pub db_path: PathBuf,
}

/*
#[derive(Serialize, Deserialize)]
pub struct TopicData {
    /// Topic name
    pub name: String,
    // Ordered list of media files
    //pub media: Vec<MediaUid>,
    // A stack of revisions, each revision is an ordered list of specific revision operations
    /// A stack of revision operations
    pub revs: Vec<RevisionOp>,
    pub owner: Option<PublicKey>,
}

impl TopicData {
    pub fn new(
        name: String,
        owner: Option<PublicKey>,
        uids: Vec<MediaUid>,
        ) -> Self {
        let mut t = Self {
            name,
            revs: vec![],
            owner,
        };
        t.add(uids);
        t
    }

    pub fn contains(&self, media: &MediaUid) -> bool {
        // debug display the list
        self.list().contains(media)
    }

    pub fn rename(&mut self, old: MediaUid, new: MediaUid) {
        if old == new || !self.contains(&old) {
            return;
        }

        self.rm(vec![old]);
        self.add(vec![new]);
    }

    pub fn add(&mut self, media: Vec<MediaUid>) {
        // First remove any media that is already in the list
        let mut media = media;
        media.retain(|x| !self.contains(x));
        self.revs.push(RevisionOp::Add(media));
    }

    pub fn rm(&mut self, media: Vec<MediaUid>) {
        // First remove any media that is already in the list
        let mut media = media;
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

#[derive(Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    pub topics: HashSet<String>,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum RevisionOp {
    Add(Vec<MediaUid>),
    Del(Vec<MediaUid>),
}
*/

/*
pub struct Topic {
    pub name: String,
    pub data: AcidJson<TopicData>,
}

pub impl Topic {
    pub fn open(name: &str) -> Topic {
        let data = AcidJson::open(name).unwrap();
        Topic {
            name: name.to_string(),
            data,
        }
    }
}
*/

/*
#[derive(Serialize, Deserialize)]
pub struct TopicRevisionHistory {
    /// Topic name
    pub topic: String,
    // A stack of revisions, each revision is an ordered list of specific revision operations
    pub revs: Vec<Vec<RevisionOp>>,
}
*/
