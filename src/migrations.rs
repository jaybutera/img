use crate::types::{Args, Index, TopicData, MediaUid};
use anyhow::anyhow;
use std::fmt::Debug;
use std::str::FromStr;
use http_types::mime::{self, Mime};
use std::path::PathBuf;
use structopt::StructOpt;
use tide::{log, Body, Request, Response, StatusCode};
use acidjson::AcidJson;
use smol::io::AsyncWriteExt;
use http_types::headers::HeaderValue;
use tide::security::{CorsMiddleware, Origin};
use async_fs::File;
use smol::io::{AsyncReadExt, BufReader};
use smol::stream::StreamExt;

use crate::get_uid;
use crate::utils::{get_topic_ids, serialize_topics, get_media_paths};

pub async fn update_media_names(root_dir: &PathBuf) -> anyhow::Result<()> {
    let json_files = get_topic_ids(root_dir).await?;
    log::info!("Found {} json files", json_files.len());

    // Change the name of the file to the hash of the first 1MB of the file
    let mut entries = smol::fs::read_dir(root_dir).await?;
    while let Some(entry) = entries.try_next().await? {
        log::info!("Processing {:?}", entry.path());
        // Filter to extensions mp4, jpg, jpeg, png
        let path = entry.path();
        // If no extension, skip
        let ext = path.extension()
            .map(|ext| ext.to_str().unwrap())
            .unwrap_or_default();

        if ext != "mp4" && ext != "jpg" && ext != "jpeg" && ext != "png" {
            continue;
        }

        let mut file = smol::fs::File::open(&path).await?;
        let mut reader = smol::io::BufReader::new(file);
        let mut buffer = vec![0; 1024 * 1024]; // 1MB buffer
        let (uid, mut buffer) = get_uid(&mut reader).await?;
        let fname = format!("{}.{}", uid, ext);

        // Rename the file
        log::info!("Renaming {:?} to {:?}", path, fname);
        let new_path = root_dir.join(&fname);
        let old_fname = path.file_name()
            .expect("old path should have a file name").to_str()
            .expect("old file name should be a string").to_string();
        smol::fs::rename(path, new_path).await?;

        // Update all json files with the new name
        for mut topic_data in serialize_topics(&json_files).await? {
            topic_data.rename(old_fname.clone(), fname.clone());
            let json_file = root_dir.join(&topic_data.name).with_extension("json");

            let raw_json = serde_json::to_vec(&topic_data)?;
            let tmp_file = json_file.with_extension("tmp");

            let mut file = smol::fs::File::create(&tmp_file).await?;
            file.write_all(&raw_json).await?;

            smol::fs::rename(tmp_file, json_file).await?;
        }
    }

    Ok(())
}

/// Generate thumbnails for all images in the root directory
pub async fn generate_thumbnails(root_dir: &PathBuf) -> anyhow::Result<()> {
    let media_files = get_media_paths(root_dir).await?;
    log::info!("Found {} media files", media_files.len());
    let thumbnail_max_size = 500;

    // Save in thumbnail directory
    let thumbnail_dir = root_dir.join("thumbnails");
    smol::fs::create_dir_all(&thumbnail_dir).await?;

    // Generate thumbnails for all images
    let mut tasks = vec![];
    for media_file in media_files {
        // Skip if file is not an image
        let ext = media_file.extension()
            .map(|ext| ext.to_str().unwrap())
            .unwrap_or_default();
        if ext != "jpg" && ext != "jpeg" && ext != "png" {
            continue;
        }
        // Skip if thumbnail already exists
        let thumbnail_file = thumbnail_dir.join(media_file.file_name().expect("media file name should exist"));
        if thumbnail_file.exists() {
            continue;
        }

        let task = crate::utils::save_thumbnail(media_file, thumbnail_dir.clone(), thumbnail_max_size);
        tasks.push(task);
    }

    for task in tasks {
        if let Err(e) = task.await {
            log::error!("Failed to generate thumbnail: {}", e);
        }
    }

    Ok(())
}
