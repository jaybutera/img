use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashSet;

pub type MediaUid = String;

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
