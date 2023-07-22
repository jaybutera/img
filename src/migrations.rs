use crate::types::{Args, Index, TopicData, MediaUid};
use anyhow::anyhow;
use std::fmt::Debug;
use std::str::FromStr;
use askama::Template;
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

pub async fn update_media_names(root_dir: &PathBuf) -> anyhow::Result<()> {
    // Get all the json files in the root directory
    let mut entries = smol::fs::read_dir(root_dir).await?;
    let mut json_files = vec![];
    while let Some(entry) = entries.try_next().await? {
        let path = entry.path();
        if path.extension().map(|ext| ext == "json").unwrap_or(false) {
            json_files.push(path);
        }
    }
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
        //reader.read_exact(&mut buffer).await?;
        //let uid = blake3::hash(&buffer).to_string();
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
        for json_file in &json_files {
            let mut topic_data: TopicData = {
                let mut file = smol::fs::File::open(json_file).await?;
                let mut raw_json = vec![];
                file.read_to_end(&mut raw_json).await?;
                serde_json::from_slice(&raw_json)?
            };

            topic_data.rename(old_fname.clone(), fname.clone());

            let raw_json = serde_json::to_vec(&topic_data)?;
            let tmp_file = json_file.with_extension("tmp");

            let mut file = smol::fs::File::create(&tmp_file).await?;
            file.write_all(&raw_json).await?;

            smol::fs::rename(tmp_file, json_file).await?;
        }
    }

    Ok(())
}
