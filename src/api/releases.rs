use flate2::read::GzDecoder;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::sync::mpsc::UnboundedSender;

use crate::api::client::{ClientError, Method, fetch_file, fetch_json};

#[derive(Debug, Deserialize, Clone)]
pub struct Asset {
    pub browser_download_url: String,
    pub name: String,
    pub content_type: String,
    pub digest: Option<String>,
    pub size: u64,
}
#[derive(Debug, Deserialize, Clone)]
pub struct Release {
    pub url: String,
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

impl Release {
    pub fn get_download_link(&self) -> Option<String> {
        self.assets
            .iter()
            .find(|asset| asset.name.ends_with(".gz"))
            .map(|asset| asset.browser_download_url.clone())
    }
    pub fn get_download_size(&self) -> Option<u64> {
        self.assets
            .iter()
            .find(|asset| asset.name.ends_with(".gz"))
            .map(|asset| asset.size)
    }
    pub fn get_checksum(&self) -> Option<String> {
        self.assets
            .iter()
            .find(|asset| asset.name.ends_with(".gz"))
            .map(|asset| asset.digest.clone())
            .flatten()
    }
}

pub async fn get_releases(
    client: &reqwest::Client,
    page: i32,
) -> Result<Vec<Release>, ClientError> {
    let url = format!(
        "https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases?page={}",
        page
    );
    let releases = fetch_json::<Vec<Release>, String>(
        &url,
        client,
        Method::Get,
        None,
        Some(vec![
            ("User-Agent", "prefix-manager"),
            ("Accept", "application/vnd.github.v3+json"),
        ]),
    )
    .await?;

    Ok(releases)
}

pub async fn download_release(
    client: &reqwest::Client,
    release: &Release,
    path: &str,
    checksum: Option<String>,
    tx: UnboundedSender<i64>,
) -> Result<(), ClientError> {
    let url = release.get_download_link().unwrap_or("".to_owned());
    let release = fetch_file(&url, client, |f| {
        let _ = tx.send(f);
    })
    .await?;

    if let Some(checksum) = checksum {
        let sha256 = Sha256::digest(&release);
        let hex = hex::encode(sha256);
        let extracted_checksum = &checksum[7..];
        if hex != extracted_checksum {
            println!("\nChecksum mismatch")
        } else {
            println!("\nChecksum verified")
        }
    } else {
        println!("\nNo checksum provided")
    }

    println!("Unpacking...");

    let gz = GzDecoder::new(&release[..]);
    let mut archive = tar::Archive::new(gz);
    archive.unpack(path)?;

    Ok(())
}
