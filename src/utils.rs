use anyhow::Result;
use std::path::PathBuf;
use blocking::unblock;
use smol::stream::StreamExt;
use crate::types::TopicData;
use smol::io::{AsyncReadExt, BufReader};
use image::{io::Reader, imageops::FilterType};
use tide::log;

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
    smol::unblock(move || {
        log::info!("Attempting to save thumbnail for {:?}", media_file);
        let img = image::open(&media_file)?;
        log::info!("Saving thumbnail for {:?}", media_file);

        let thumbnail = img.thumbnail(thumbnail_max_size, thumbnail_max_size);
            //.rotate180();

        let mut output_path = thumbnail_dir;
        output_path.push(media_file.file_name().unwrap());

        thumbnail.save(output_path)
    }).await
    .map_err(|e| anyhow::anyhow!("Error saving thumbnail: {:?}", e))
}
