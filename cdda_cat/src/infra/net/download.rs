use std::fs::File;
use std::io::Write;
use std::{cmp::min, path::PathBuf};

use anyhow::{anyhow, Error};
use futures_util::StreamExt;
use indicatif::ProgressBar;
use reqwest::{Client, Response};

#[derive(Debug, Default, Clone)]
pub struct DownloadInfo {
    pub client: Client,
    pub url: String,
    pub filepath: PathBuf,
}

impl DownloadInfo {
    pub fn new(url: &str, filepath: PathBuf) -> Self {
        Self {
            client: Client::new(),
            url: url.to_string(),
            filepath,
        }
    }
}

impl DownloadInfo {
    pub async fn download_file(
        &self,
        progress_bar: &ProgressBar,
        response: Response,
    ) -> Result<(), Error> {
        let total_length = response
            .content_length()
            .ok_or(anyhow!("Failed to get content length from '{}'", &self.url))?;
        let mut file = File::create(&self.filepath).or(Err(anyhow!(
            "Failed to create file '{}'",
            self.filepath.display()
        )))?;

        stream_download(&mut file, progress_bar, total_length, response).await
    }
}

async fn stream_download(
    file: &mut File,
    progress_bar: &ProgressBar,
    total_length: u64,
    response: Response,
) -> Result<(), Error> {
    let mut downloaded = 0;
    let mut stream = response.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(anyhow!("Error while downloading file")))?;
        file.write_all(&chunk)
            .or(Err(anyhow!("Error while writing to file")))?;
        let new = min(downloaded + (chunk.len() as u64), total_length);
        downloaded = new;
        progress_bar.set_position(new);
    }

    Ok(())
}
