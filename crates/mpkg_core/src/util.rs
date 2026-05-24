use std::{io::Cursor, path::Path, time::Duration};

use futures_util::StreamExt;
use indicatif::ProgressBar;
use reqwest::Client;
use tar::Archive;
use zip::ZipArchive;

use crate::{error::Error, lang_primitives::ArchiveType};

pub fn extract_archive(
    data: &[u8],
    archive_type: ArchiveType,
    build_dir: &Path,
    headless: bool,
) -> Result<(), Error> {
    if !headless {
        println!("Extracting...");
    }

    match archive_type {
        ArchiveType::Zip => {
            let data = Cursor::new(data);

            let mut zip_archive = ZipArchive::new(data)?;

            zip_archive.extract(build_dir)?;
        }
        ArchiveType::Tar => {
            let mut tar_archive = Archive::new(data);

            tar_archive.unpack(build_dir)?;
        }
        _ => (),
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
