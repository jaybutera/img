use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
