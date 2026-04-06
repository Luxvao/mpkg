use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use archive::{ArchiveExtractor, ArchiveFormat};
use futures_util::StreamExt;
use indicatif::ProgressBar;
use reqwest::Client;

use crate::error::Error;

pub fn extract_archive(
    data: &[u8],
    archive_format: ArchiveFormat,
    build_dir: &Path,
    headless: bool,
) -> Result<(), Error> {
    if !headless {
        println!("Extracting...");
    }

    let extractor = ArchiveExtractor::new();

    let files = extractor.extract(data, archive_format)?;

    for file in files {
        let mut fixed_path = PathBuf::from(build_dir);
        fixed_path.push(format!("./{}", &file.path));

        if !headless {
            println!("Inflating `{}`", file.path);
        }

        if file.is_directory {
            std::fs::create_dir_all(&fixed_path)?;
            continue;
        }

        if let Some(parent) = fixed_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(fixed_path, file.data)?;
    }

    Ok(())
}

pub async fn download_with_progress(url: &str) -> Result<Vec<u8>, Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(3600))
        .build()?;

    let resp = client.get(url).send().await?;

    let size = resp.content_length().unwrap_or(0);

    let pb = ProgressBar::new(size);

    let mut stream = resp.bytes_stream();

    let mut body: Vec<u8> = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;

        pb.inc(chunk.len() as u64);

        body.extend_from_slice(&chunk);
    }

    Ok(body)
}
