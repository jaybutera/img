use anyhow::Result;
use std::path::PathBuf;
use smol::stream::StreamExt;
use crate::types::{PublicKey, TopicData, Index};
use std::collections::HashSet;
use smol::io::{BufWriter, AsyncRead, AsyncWriteExt, AsyncReadExt, BufReader};
use http_types::mime::Mime;
use std::str::FromStr;
use anyhow::anyhow;
use acidjson::AcidJson;
//use rand::rngs::OsRng;
use smol::fs::File;

/// Get all topic file paths in the root directory
pub async fn get_topic_ids(root_dir: &PathBuf) -> Result<Vec<PathBuf>> {
    // Get all the json files in the root directory
    let mut entries = smol::fs::read_dir(root_dir).await?;
    let mut json_files = vec![];
    while let Some(entry) = entries.try_next().await? {
        let path = entry.path();
        if path.extension().map(|ext| ext == "json").unwrap_or(false) {
            json_files.push(path);
        }
    }

    Ok(json_files)
}

/// Convert topic paths into TopicData structs
pub async fn serialize_topics(topics: &Vec<PathBuf>) -> Result<Vec<TopicData>> {
    let mut topic_data = vec![];
    for topic in topics {
        let mut file = File::open(&topic).await?;
        let mut raw_json = vec![];
        file.read_to_end(&mut raw_json).await?;
        let topic: TopicData = serde_json::from_slice(&raw_json)?;
        topic_data.push(topic);
    }

    Ok(topic_data)
}

/// Get paths of all the media files in the root directory
pub async fn get_media_paths(root_dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut entries = smol::fs::read_dir(root_dir).await?;
    let mut media_files = vec![];
    while let Some(entry) = entries.try_next().await? {
        let path = entry.path();
        if path.extension().map(|ext| {
            ext == "mp4" || ext == "jpg" || ext == "jpeg" || ext == "png"
        }).unwrap_or(false) {
            media_files.push(path);
        }
    }

    Ok(media_files)
}

pub async fn save_thumbnail(
    media_file: PathBuf,
    thumbnail_dir: PathBuf,
    thumbnail_max_size: u32,
) -> anyhow::Result<()> {
    let media_file1 = media_file.clone();
    smol::unblock(move || {
        let img = image::open(&media_file)?;

        let thumbnail = img.thumbnail(thumbnail_max_size, thumbnail_max_size);

        let mut output_path = thumbnail_dir;
        output_path.push(media_file.file_name().expect("Should be a filename when saving thumbnail"));

        // Save thumbnail to a temporary file
        //let temp_output_path = output_path.with_extension("temp.jpg");
        thumbnail.save(&output_path)?;

        // Load EXIF data from original image
        let metadata = rexiv2::Metadata::new_from_path(&media_file)?;
        
        // Load the temporary thumbnail image and apply the metadata
        let mut output_metadata = rexiv2::Metadata::new_from_path(&output_path)?;
        output_metadata.set_orientation(metadata.get_orientation());
        output_metadata.save_to_file(&output_path)
            .map_err(|e| anyhow::anyhow!("Error saving thumbnail metadata: {:?}", e))

        // Rename the temp thumbnail to the final thumbnail path
        //std::fs::rename(temp_output_path, &output_path)?;
    }).await
    .map_err(|e| anyhow::anyhow!("Error saving thumbnail [{media_file1:?}]: {:?}", e))
}

/// List all index files in the root/indexes directory
pub async fn get_index_paths(root_dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let index_dir = root_dir.join("indexes");
    if !index_dir.exists() {
        smol::fs::create_dir(&index_dir).await?;
    }
    let mut entries = smol::fs::read_dir(index_dir).await?;
    let mut index_files = vec![];
    while let Some(entry) = entries.try_next().await? {
        let path = entry.path();
        if path.extension().map(|ext| ext == "json").unwrap_or(false) {
            index_files.push(path);
        }
    }

    Ok(index_files)
}

/// Search all /indexes/*.json files for the existence of the topic
pub async fn get_tags_for_topic(
    root_dir: &PathBuf,
    topic: &String,
) -> Result<HashSet<String>> {
    let index_paths = get_index_paths(root_dir).await?;
    let mut tags = HashSet::new();
    for index_path in index_paths {
        let mut file = File::open(&index_path).await?;
        let mut raw_json = vec![];
        file.read_to_end(&mut raw_json).await?;

        let index: Index = serde_json::from_slice(&raw_json)?;
        if index.topics.contains(topic) {
            tags.insert(index.name);
        }
    }

    Ok(tags)
}

pub async fn add_tag_for_topic(
    root_dir: &PathBuf,
    topic: String,
    tag: String,
) -> Result<()> {
    let index_paths = get_index_paths(root_dir).await?;

    // If root_dir/indexes directory doesnt exist, create it
    let mut tag_path = root_dir.join("indexes");
    if !tag_path.exists() {
        smol::fs::create_dir(&tag_path).await?;
    }

    let mut tag_path = root_dir.join("indexes");
    tag_path.push(format!("{}.json", tag));

    // Read the index if it exists, otherwise create it
    let mut index = if tag_path.exists() {
        let mut file = File::open(&tag_path).await?;
        let mut raw_json = vec![];
        file.read_to_end(&mut raw_json).await?;
        serde_json::from_slice(&raw_json)?
    } else {
        Index {
            name: tag,
            topics: HashSet::new(),
        }
    };
    index.topics.insert(topic.clone());

    // Write to a temporary {topic}.temp.json and then rename
    let temp_tag_path = tag_path.with_extension(format!("{}.temp.json", topic));
    let mut file = File::create(&temp_tag_path).await?;
    file.write_all(serde_json::to_string(&index)?.as_bytes()).await?;
    std::fs::rename(temp_tag_path, &tag_path.with_extension("json"))?;

    Ok(())
}

pub async fn rm_tag_for_topic(
    root_dir: &PathBuf,
    topic: String,
    tag: String,
) -> Result<()> {
    let index_paths = get_index_paths(root_dir).await?;

    let mut tag_path = root_dir.join("indexes");
    tag_path.push(format!("{}.json", tag));

    // Read the index if it exists, otherwise fail
    let mut index: Index = if tag_path.exists() {
        let mut file = File::open(&tag_path).await?;
        let mut raw_json = vec![];
        file.read_to_end(&mut raw_json).await?;
        serde_json::from_slice(&raw_json)?
    } else {
        return Err(anyhow::anyhow!("Tag does not exist"));
    };

    index.topics.remove(&topic);

    // If topics is empty remove the index file
    if index.topics.is_empty() {
        smol::fs::remove_file(&tag_path).await?;
    } else {
        // Write to a temporary {topic}.temp.json and then rename
        let temp_tag_path = tag_path.with_extension(format!("{}.temp.json", topic));
        let mut file = File::create(&temp_tag_path).await?;
        file.write_all(serde_json::to_string(&index)?.as_bytes()).await?;
        std::fs::rename(temp_tag_path, &tag_path.with_extension("json"))?;
    }

    Ok(())
}

/// Uses the buffer to read the first 1MB of the file and hash it to get the uid
pub async fn get_uid<T>(reader: &mut BufReader<T>) -> anyhow::Result<(String, Vec<u8>)>
where T: AsyncRead + Unpin {
    let mut buffer = vec![0; 1024 * 1024]; // 1MB buffer

    // read exactly 1MB or the whole buffer
    let mut n = reader.read(&mut buffer).await?;
    while n < buffer.len() {
        let n2 = reader.read(&mut buffer[n..]).await?;
        if n2 == 0 {
            break;
        }
        n += n2;
    }

    // Hash the first 1MB for the uid
    let uid = blake3::hash(&buffer).to_string();

    Ok((uid, buffer))
}

/// Saves media to the filesystem and return the filename which is the hash of first 1MB of 
/// the file as the uid, and the extension of the file.
/*
pub async fn save_media<T>(
    mut reader: BufReader<T>,
    root_dir: &PathBuf,
    ext: &str,
    thumbnail_sender: smol::channel::Sender<PathBuf>,
) -> anyhow::Result<String> {
    let (uid, mut buffer) = get_uid(&mut reader).await?;

    // Generate path
    let image_fname = format!("{}.{}", uid, ext);
    let image_path = root_dir.join(&image_fname);

    // Read and write the file
    if !image_path.exists() {
        let mut image_file = File::create(&image_path).await?;

        // Write the first chunk
        image_file.write_all(&buffer).await?;
        image_file.flush().await?;

        // Loop the rest
        while let Ok(bytes_read) = reader.read(&mut buffer).await {
            if bytes_read == 0 {
                break;
            }

            let chunk = &buffer[..bytes_read];
            image_file.write_all(chunk).await?;
            image_file.flush().await?;
        }

        // Save thumbnail
        thumbnail_sender.send(image_path.clone()).await?;
    }

    Ok(image_fname)
}
*/



// Separate --------------------------------------------------------------------
// -----------------------------------------------------------------------------

/*
use actix_web::web::Payload;
use smol::io::AsyncBufRead;
use std::pin::Pin;
use std::task::{Context, Poll};
use bytes::{Bytes, Buf};
//use futures::StreamExt;

pub struct PayloadReader {
    payload: Payload,
    buffer: Bytes,
}

impl PayloadReader {
    pub fn new(payload: Payload) -> Self {
        Self {
            payload,
            buffer: Bytes::new(),
        }
    }
}

impl AsyncRead for PayloadReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        if self.buffer.is_empty() {
            match Pin::new(&mut self.payload).poll_next(cx) {
                Poll::Ready(Some(Ok(chunk))) => {
                    self.buffer = chunk;
                },
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e))),
                Poll::Ready(None) => return Poll::Ready(Ok(0)),
                Poll::Pending => return Poll::Pending,
            }
        }

        let len = std::cmp::min(buf.len(), self.buffer.len());
        buf[0..len].copy_from_slice(&self.buffer[0..len]);
        self.buffer.advance(len);

        Poll::Ready(Ok(len))
    }
}

impl AsyncBufRead for PayloadReader {
    fn poll_fill_buf(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<&[u8]>> {
        if self.buffer.is_empty() {
            match Pin::new(&mut self.payload).poll_next(cx) {
                Poll::Ready(Some(Ok(chunk))) => {
                    self.buffer = chunk;
                },
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e))),
                Poll::Ready(None) => return Poll::Ready(Ok(&[])),
                Poll::Pending => return Poll::Pending,
            }
        }

        Poll::Ready(Ok(&self.buffer))
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        self.buffer.advance(amt);
    }
}
*/

use blake3::{Hasher, OutputReader};
//use hex_literal::hex;
use rand::Rng;

fn rand_string() -> String {
    let mut rng = rand::thread_rng();
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             abcdefghijklmnopqrstuvwxyz\
                             0123456789";
    // 34 to guanrantee no conflicts with other files
    const PASSWORD_LEN: usize = 34;
    let password: String = (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    
    password
}

pub async fn save_file(
    root_dir: &PathBuf,
    mut payload: actix_web::web::Payload,
    ext: &str,
    thumbnail_sender: smol::channel::Sender<PathBuf>,
) -> anyhow::Result<String> {
    let mut hasher = Hasher::new();
    // First give it a random temp name
    let rand_name = format!("{}.tmp", rand_string());
    let file = File::create(&rand_name).await?;

    // TODO limit chunk size
    let mut buf_writer = BufWriter::new(file);
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        hasher.update(&chunk);
        buf_writer.write_all(&chunk).await?;
    }

    buf_writer.flush().await?;

    let mut hash_output = [0; 32];
    hasher.finalize_xof().fill(&mut hash_output);

    let uid = hex::encode(hash_output);

    // Rename file
    let image_fname = format!("{}.{}", uid, ext);
    let image_path = root_dir.join(&image_fname);
    smol::fs::rename(&rand_name, &image_path).await?;

    // Save thumbnail
    thumbnail_sender.send(image_path.clone()).await?;

    Ok(image_fname)
}


pub fn mime_and_ext(
    req: &actix_web::HttpRequest,
) -> Result<(Mime, String), actix_web::error::Error> {
    // Get Content-Type header
    let mime = req
        .headers()
        .get("Content-Type")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("No content type"))?
        .to_str()
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid content type"))?;
    let mime = Mime::from_str(mime)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid content type"))?;
    let ext = mime.subtype().to_string();
    Ok((mime, ext))
}

pub fn is_valid_media(mime: &Mime) -> Result<(), actix_web::error::Error> {
    if mime.basetype() != "image" && mime.basetype() != "video" {
        return Err(actix_web::error::ErrorBadRequest(format!(
            "Invalid content type {}",
            mime
        )));
    }
    Ok(())
}

pub fn get_topic_owner(
    topic_path: &PathBuf,
) -> Result<PublicKey, anyhow::Error> {
    let topic = topic_path.file_stem().unwrap().to_str().unwrap();
    if topic_path.exists() {
        let topic_file: AcidJson<TopicData> = AcidJson::open(&topic_path)
            .map_err(|e| anyhow!("{}", e))?;
        let td = topic_file.read();
        if let Some(ref owner) = td.owner {
            Ok(owner.clone())
        } else {
            Err(anyhow!("Topic {} is not owned", topic))
        }
    } else {
        Err(anyhow!("Topic {} does not exist", topic))
    }
}
