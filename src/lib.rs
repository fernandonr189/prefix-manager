use tokio::sync::mpsc::UnboundedSender;

use crate::api::{client::ClientError, releases::Release};

pub mod api;

pub struct PrefixManager {
    client: reqwest::Client,
}

impl PrefixManager {
    pub fn new(client: reqwest::Client) -> Self {
        PrefixManager { client: client }
    }
    pub fn new_with_default_client() -> Self {
        PrefixManager {
            client: reqwest::Client::new(),
        }
    }
    pub async fn get_releases(&self, page: i32) -> Result<Vec<Release>, ClientError> {
        api::releases::get_releases(&self.client, page).await
    }
    pub async fn download_release(
        &self,
        release: &Release,
        path: &str,
        checksum: Option<String>,
        tx: UnboundedSender<i64>,
    ) -> Result<(), ClientError> {
        api::releases::download_release(&self.client, release, path, checksum, tx).await
    }
}
