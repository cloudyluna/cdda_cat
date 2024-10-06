use std::path::PathBuf;

use crate::infra::net::download::DownloadInfo;
use crate::infra::ui::cli::progress_bar::ProgressBarInfo;
use anyhow::anyhow;
use anyhow::Error;
use indicatif::{ProgressBar, ProgressStyle};

pub async fn download_archive(from_url: &str, to_path: &PathBuf) -> Result<(), Error> {
    let mut download_info = DownloadInfo::new(from_url, to_path.to_owned());
    let progress_bar_info = ProgressBarInfo::new(
        &format!("Downloading from {}", &download_info.url),
        &format!(
            "Finished downloading from {} to {}",
            &download_info.url,
            to_path.display()
        ),
    );

    if !download_info.filepath.exists() {
        println!("We checked for existing archive file, but couldn't find any.");
        println!("Proceed to download new archive.");
        download_info
            .download_with_progress_bar(&progress_bar_info)
            .await
    } else {
        println!("Archive file already exists.");
        Ok(())
    }
}

impl DownloadInfo {
    async fn download_with_progress_bar(
        &mut self,
        progress_bar_info: &ProgressBarInfo,
    ) -> Result<(), Error> {
        let response = self
            .client
            .get(&self.url)
            .send()
            .await
            .or(Err(anyhow!("Failed to GET from '{}'", self.url.clone())))?;

        let total_length = response
            .content_length()
            .ok_or(anyhow!("Failed to get content length from '{}'", &self.url))?;

        let progress_bar = ProgressBar::new(total_length);
        progress_bar.set_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
    .unwrap()
        .progress_chars("#>-"));
        progress_bar.set_message(progress_bar_info.pre_message.to_string());
        self.download_file(&progress_bar, response).await?;
        progress_bar.set_message(progress_bar_info.post_message.to_string());

        Ok(())
    }
}
