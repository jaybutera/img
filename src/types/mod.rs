pub mod mimes;

use std::path::PathBuf;
use structopt::StructOpt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashSet;
use thiserror::Error;
use acidjson::AcidJson;
use anyhow::anyhow;
use log::info;

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct HashVal([u8; 32]);

pub type MediaUid = String;

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

//pub type PublicKey = Vec<u8>;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey([u8; 32]);

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let b64_str = base64::encode(&self.0);
        serializer.serialize_str(&b64_str)
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let b64_str = String::deserialize(deserializer)?;
        let bytes = base64::decode(&b64_str).map_err(serde::de::Error::custom)?;

        let mut array = [0; 32];
        if bytes.len() != array.len() {
            return Err(serde::de::Error::custom("Expected length 32"));
        }
        array.copy_from_slice(&bytes);
        Ok(PublicKey(array))
    }
}

impl PublicKey {
    /*
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn from_slice(bytes: &[u8]) -> Self {
        let mut array = [0; 32];
        array.copy_from_slice(&bytes);
        Self(array)
    }
    */

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }
    pub fn to_string(&self) -> String {
        base64::encode(&self.0)
    }
}

impl Into<PublicKey> for String {
    fn into(self) -> PublicKey {
        let bytes = base64::decode(&self).unwrap();
        let mut array = [0; 32];
        array.copy_from_slice(&bytes);
        PublicKey(array)
    }
}

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
    // TODO this should open a read/write guard from acidjson?
    pub async fn open_or_create(path: &PathBuf, owner: Option<PublicKey>) -> anyhow::Result<TopicData> {
        if !path.exists() {
            let td = TopicData {
                name: path.file_name().unwrap().to_str().unwrap().to_string(),
                revs: vec![],
                owner,
            };
            smol::fs::write(
                path,
                serde_json::to_vec_pretty(&td)
                    .expect("Failed to serialize topic data")).await?;

            Ok(td)
        } else {
            //let td: AcidJson<TopicData> = AcidJson::open(&path)
            //    .map_err(|e| AnyError::from(anyhow!("{}", e)))?;
            let file = smol::fs::read(path).await?;
            let td: TopicData = serde_json::from_slice(&file)?;
            Ok(td)
        }
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
