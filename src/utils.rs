use anyhow::Result;
use std::path::PathBuf;
use smol::stream::StreamExt;
use crate::types::{TopicData, Index};
use std::collections::HashSet;
use smol::io::{AsyncRead, AsyncWriteExt, AsyncReadExt, BufReader};
//use rand::rngs::OsRng;
use std::fs::File;

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
        let mut file = smol::fs::File::open(&topic).await?;
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
        let mut file = smol::fs::File::open(&index_path).await?;
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
        let mut file = smol::fs::File::open(&tag_path).await?;
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
    let mut file = smol::fs::File::create(&temp_tag_path).await?;
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
        let mut file = smol::fs::File::open(&tag_path).await?;
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
        let mut file = smol::fs::File::create(&temp_tag_path).await?;
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

