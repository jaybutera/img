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

/*
pub async fn save_thumbnail(
    media_file: PathBuf,
    thumbnail_dir: PathBuf,
    thumbnail_max_size: u32,
) -> anyhow::Result<()> {
    smol::unblock(move || {
        let img = image::open(&media_file)?;

        let thumbnail: i32 = img.thumbnail(thumbnail_max_size, thumbnail_max_size);
            //.rotate180();

        let mut output_path = thumbnail_dir;
        output_path.push(media_file.file_name().unwrap());

        thumbnail.save(output_path)
    }).await
    .map_err(|e| anyhow::anyhow!("Error saving thumbnail: {:?}", e))
}
*/
pub async fn save_thumbnail(
    media_file: PathBuf,
    thumbnail_dir: PathBuf,
    thumbnail_max_size: u32,
) -> anyhow::Result<()> {
    smol::unblock(move || {
        let img = image::open(&media_file)?;

        let thumbnail = img.thumbnail(thumbnail_max_size, thumbnail_max_size);

        let mut output_path = thumbnail_dir;
        output_path.push(media_file.file_name().unwrap());

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
    .map_err(|e| anyhow::anyhow!("Error saving thumbnail: {:?}", e))
}

