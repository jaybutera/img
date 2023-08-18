use std::path::PathBuf;
use structopt::StructOpt;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use tide::log;

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

pub type PublicKey = Vec<u8>;

#[derive(Serialize, Deserialize)]
pub struct VerificationPayload {
    pub public_key: PublicKey,
    pub signature: Vec<u8>,
}

#[derive(Clone)]
pub struct ServerState {
    pub args: Args,
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
