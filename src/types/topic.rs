use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashSet;
use crate::PublicKey;

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
        let media = media;
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

/// Topic ID associated with first 32 bits of public key
pub struct OwnedTopicId {
    pub topic: String,
    pub owner_id: String,
}

impl OwnedTopicId {
    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}

impl Serialize for OwnedTopicId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let owner_id = self.owner_id.clone();
        let topic = self.topic.clone();
        serializer.serialize_str(&(topic + "." + owner_id.as_str()))
    }
}

impl<'de> Deserialize<'de> for OwnedTopicId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Parse topic and owner_id from string delimited by '.'
        let s = String::deserialize(deserializer)?;
        let mut split = s.split('.');
        let topic = split.next().ok_or(serde::de::Error::custom("Expected topic"))?;
        let owner_id = split.next().ok_or(serde::de::Error::custom("Expected owner_id"))?;

        Ok(OwnedTopicId {
            topic: topic.to_string(),
            owner_id: owner_id.to_string(),
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum RevisionOp {
    Add(Vec<MediaUid>),
    Del(Vec<MediaUid>),
}
