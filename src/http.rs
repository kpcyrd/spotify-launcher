use crate::errors::*;
use bytes::Bytes;
use reqwest::Response;
use std::time::Duration;
use tokio::time;

pub struct Client {
    client: reqwest::Client,
    timeout: Option<Duration>,
}

impl Client {
    #[inline]
    pub fn new(timeout: Option<u64>) -> Result<Client> {
        let (connect_timeout, timeout) = if let Some(timeout) = timeout {
            if timeout > 0 {
                let timeout = Duration::from_secs(timeout);
                (Some(timeout), Some(timeout))
            } else {
                (None, None)
            }
        } else {
            (Some(Duration::from_secs(10)), Some(Duration::from_secs(30)))
        };

        let mut builder = reqwest::ClientBuilder::new()
            .user_agent(format!("spotify-launcher/{}", env!("CARGO_PKG_VERSION")))
            .redirect(reqwest::redirect::Policy::limited(8));

        if let Some(timeout) = connect_timeout {
            builder = builder.connect_timeout(timeout);
        }

        let client = builder.build().context("Failed to create http client")?;

        Ok(Client { client, timeout })
    }

    async fn send_get(&self, url: &str) -> Result<Response> {
        let future = async {
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
        let resp = self.send_get(url).await?;

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

    pub async fn fetch_stream(&self, url: &str) -> Result<Download> {
        debug!("Downloading {:?}...", url);
        let resp = self.send_get(url).await?;

        let total = resp.content_length().unwrap_or(0);

        Ok(Download {
            resp,
            timeout: self.timeout,
            progress: 0,
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
