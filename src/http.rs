use crate::errors::*;
use bytes::Bytes;
use std::time::Duration;

pub struct Client {
    client: reqwest::Client,
}

impl Client {
    #[inline]
    pub fn new() -> Result<Client> {
        let client = reqwest::ClientBuilder::new()
            .user_agent(format!("spotify-launcher/{}", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(7))
            .timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::limited(8))
            .build()
            .context("Failed to create http client")?;
        Ok(Client { client })
    }

    pub async fn fetch(&self, url: &str) -> Result<Vec<u8>> {
        debug!("Downloading {:?}...", url);
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to send http request")?;

        let status = resp.status();
        if !status.is_success() {
            bail!("Unexpected http status code: {:?}", status);
        }

        let body = resp.bytes().await.context("Failed to read http response")?;
        debug!("Downloaded {} bytes", body.len());
        Ok(body.to_vec())
    }

    pub async fn fetch_stream(&self, url: &str) -> Result<Download> {
        debug!("Downloading {:?}...", url);
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to send http request")?;

        let status = resp.status();
        if !status.is_success() {
            bail!("Unexpected http status code: {:?}", status);
        }

        let total = resp.content_length().unwrap_or(0);

        Ok(Download {
            resp,
            progress: 0,
            total,
        })
    }
}

pub struct Download {
    resp: reqwest::Response,
    pub progress: u64,
    pub total: u64,
}

impl Download {
    pub async fn chunk(&mut self) -> Result<Option<Bytes>> {
        let bytes = self.resp.chunk().await?;
        if let Some(bytes) = bytes {
            self.progress += bytes.len() as u64;
            Ok(Some(bytes))
        } else {
            Ok(None)
        }
    }
}
