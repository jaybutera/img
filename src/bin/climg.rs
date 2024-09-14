use std::env;
//use smol::fs::File;
use std::fs::File;
use std::path::PathBuf;
//use smol::io::{BufWriter, AsyncRead, AsyncWriteExt, AsyncBufReadExt, BufReader};
use std::io::{BufWriter, BufReader, BufRead};
use blake3::Hasher;
use color_eyre::Result;
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

/*
pub async fn save_file(
    root_dir: &PathBuf,
    //mut payload: actix_web::web::Payload,
    mut payload: actix_multipart::Field,
    ext: String,
    thumbnail_sender: smol::channel::Sender<PathBuf>,
) -> Result<String, ServerErr> {
    let mut hasher = Hasher::new();
    // First give it a random temp name
    let rand_name = format!("{}.tmp", rand_string());
    let file = File::create(&rand_name).await?;
    log::info!("Saving file to {}", rand_name);

    // TODO limit chunk size
    let mut buf_writer = BufWriter::new(file);
    while let Some(chunk) = payload.next().await {
        let chunk = chunk
            .map_err(|e| anyhow::anyhow!("Error reading payload: {}", e))?;
        hasher.update(&chunk);
        buf_writer.write_all(&chunk).await?;
    }

    log::info!("Flushing file {}", rand_name);
    buf_writer.flush().await?;

    let mut hash_output = [0; 32];
    hasher.finalize_xof().fill(&mut hash_output);

    let uid = hex::encode(hash_output);

    // Check if file already exists
    let image_fname = format!("{}.{}", uid, ext);
    let image_path = root_dir.join(&image_fname);
    if image_path.exists() {
        return Ok(image_fname);
        //return Err(ServerErr::CustomError(anyhow!("File already exists".to_string())));
    }

    // Rename file
    smol::fs::rename(&rand_name, &image_path).await?;

    // Save thumbnail
    thumbnail_sender.send(image_path.clone()).await
        .map_err(|e| ServerErr::CustomError(anyhow!("Error sending thumbnail on channel: {}", e)))?;

    Ok(image_fname)
}
*/

async fn save_file(
    dest_dir: &PathBuf,
    source_file: &PathBuf,
) -> Result<()> {
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
                //let file_name = path.file_name().unwrap().to_str().unwrap();
                //let dest_path = PathBuf::from(dest).join(file_name);
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
