use std::env;
//use smol::fs::File;
use std::fs::File;
use std::path::PathBuf;
//use smol::io::{BufWriter, AsyncRead, AsyncWriteExt, AsyncBufReadExt, BufReader};
use std::io::{BufWriter, BufReader, BufRead};
use blake3::Hasher;
use color_eyre::Result;
use rand::Rng;

async fn save_file(
    dest_dir: &PathBuf,
    source_file: &PathBuf,
) -> Result<()> {
    let ext = source_file.extension().unwrap_or(std::ffi::OsStr::new("")).to_str().unwrap();
    if ext != "jpg" {
        println!("Skipping {:?}", source_file);
        return Ok(());
    }


    let mut hasher = Hasher::new();

    // First give it a random temp name
    //let rand_name = format!("{}.tmp", rand_string());
    //let file = File::create(&rand_name).await?;
    //log::info!("Saving file to {}", rand_name);

    //let mut buf_writer = BufWriter::new(file);

    const CAP: usize = 1024 * 1024;
    //let file = File::open(source_file).await?;
    let file = File::open(source_file)?;
    let mut reader = BufReader::with_capacity(CAP, file);

    loop {
        let length = {
            //let buffer = reader.fill_buf().await?;
            let buffer = reader.fill_buf()?;
            hasher.update(&buffer);
            buffer.len()
        };
        if length == 0 {
            break;
        }
        reader.consume(length);
    }


    let mut hash_output = [0; 32];
    hasher.finalize_xof().fill(&mut hash_output);
    let uid = hex::encode(hash_output);
    let image_fname = format!("{}.{}", uid, "jpg");
    let image_path = dest_dir.join(&image_fname);
    if image_path.exists() {
        return Ok(());
    }

    smol::fs::copy(&source_file, &image_path).await?;
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: climg <source> <dest>");
        std::process::exit(1);
    }
    let (source, dest) = (&args[1], &args[2]);

    smol::block_on(async {
        if std::fs::metadata(source).unwrap().is_dir() {
            let dest_dir = PathBuf::from(dest);
            for entry in std::fs::read_dir(source).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                println!("Processing {:?}", path);
                save_file(&dest_dir, &path).await.unwrap();
            }
            return;
        }
        else {
            let source_file = PathBuf::from(source);
            let dest_dir = PathBuf::from(dest);
            save_file(&dest_dir, &source_file).await.unwrap();
        }
    });
}
