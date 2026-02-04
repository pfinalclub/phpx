use crate::error::{Error, Result};
use reqwest::Client;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct Downloader {
    client: Client,
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn download_file(&self, url: &str, destination: &PathBuf) -> Result<()> {
        tracing::info!("Downloading from {} to {:?}", url, destination);

        // 确保目标目录存在
        if let Some(parent) = destination.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(Error::Network(response.error_for_status().unwrap_err()));
        }

        let content = response.bytes().await?;
        
        let mut file = File::create(destination).await?;
        file.write_all(&content).await?;
        file.flush().await?;

        tracing::info!("Download completed successfully");
        Ok(())
    }

    pub async fn download_file_with_progress(
        &self,
        url: &str,
        destination: &PathBuf,
    ) -> Result<()> {
        // TODO: 实现带进度条的下载
        self.download_file(url, destination).await
    }
}