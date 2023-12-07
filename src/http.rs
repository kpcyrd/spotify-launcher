use crate::errors::*;
use bytes::Bytes;
use reqwest::{
    header::{HeaderMap, HeaderValue, RANGE},
    Response, StatusCode,
};
use std::time::Duration;
use tokio::time;

pub struct Client {
    client: reqwest::Client,
    timeout: Option<Duration>,
}

impl Client {
    pub fn new(timeout: Option<u64>) -> Result<Client> {
        let client = reqwest::ClientBuilder::new()
            .user_agent(format!("spotify-launcher/{}", env!("CARGO_PKG_VERSION")))
            .redirect(reqwest::redirect::Policy::limited(8))
            .build()
            .context("Failed to create http client")?;

        let timeout = match timeout {
            Some(0) => None,
            Some(secs) => Some(Duration::from_secs(secs)),
            None => Some(Duration::from_secs(30)),
        };

        Ok(Client { client, timeout })
    }

    async fn send_get(&self, url: &str, offset: Option<u64>) -> Result<Response> {
        let future = async {
            let mut headers = HeaderMap::new();
            if let Some(offset) = offset {
                headers.insert(RANGE, HeaderValue::from_str(&format!("bytes={offset}-"))?);
            }

            let resp = self
                .client
                .get(url)
                .headers(headers)
                .send()
                .await
                .context("Failed to send http request")?;

            let status = resp.status();
            if !status.is_success() {
                bail!("Unexpected http status code: {:?}", status);
            }

            Ok(resp)
        };
        if let Some(timeout) = self.timeout {
            time::timeout(timeout, future)
                .await
                .context("Request timed out")?
        } else {
            future.await
        }
    }

    pub async fn fetch(&self, url: &str) -> Result<Vec<u8>> {
        debug!("Fetching {:?}...", url);
        let resp = self.send_get(url, None).await?;

        let body = resp.bytes();
        let body = if let Some(timeout) = self.timeout {
            time::timeout(timeout, body)
                .await
                .context("Reading http response timed out")?
        } else {
            body.await
        }
        .context("Failed to read http response")?;

        debug!("Fetched {} bytes", body.len());
        Ok(body.to_vec())
    }

    pub async fn fetch_stream(&self, url: &str, offset: Option<u64>) -> Result<Download> {
        debug!("Downloading {:?}...", url);
        let resp = self.send_get(url, offset).await?;

        if offset.is_some() && resp.status() != StatusCode::PARTIAL_CONTENT {
            bail!("Download server does not support resumption");
        }

        let progress = offset.unwrap_or(0);
        let total = resp.content_length().unwrap_or(0) + progress;

        Ok(Download {
            resp,
            timeout: self.timeout,
            progress,
            total,
        })
    }
}

pub struct Download {
    resp: reqwest::Response,
    timeout: Option<Duration>,
    pub progress: u64,
    pub total: u64,
}

impl Download {
    pub async fn chunk(&mut self) -> Result<Option<Bytes>> {
        let future = self.resp.chunk();
        let bytes = if let Some(timeout) = self.timeout {
            if let Ok(bytes) = time::timeout(timeout, future).await {
                bytes?
            } else {
                bail!("Download timed out due to inactivity");
            }
        } else {
            future.await?
        };
        if let Some(bytes) = bytes {
            self.progress += bytes.len() as u64;
            Ok(Some(bytes))
        } else {
            Ok(None)
        }
    }
}
