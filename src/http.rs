use std::time::Duration;

use bytes::Bytes;
use futures_util::stream::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_LENGTH, RANGE};
use reqwest::Response;
use tokio::time;

use crate::errors::*;

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

    async fn send_head(&self, url: &str) -> Result<Response> {
        let future = async {
            let resp = self
                .client
                .head(url)
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
        let total = self
            .send_head(url)
            .await?
            .headers()
            .get(CONTENT_LENGTH)
            .map(|val| val.to_str())
            .context("missing CONTENT_LENGTH")?
            .map(str::parse)??;
        debug!("{total} bytes to download");

        Ok(Download {
            url: url.to_owned(),
            client: Client::new(self.timeout.map(|duration| duration.as_secs()))?,
            timeout: self.timeout,
            progress: 0,
            total,
        })
    }
}

pub struct Download {
    url: String,
    client: Client,
    timeout: Option<Duration>,
    pub progress: u64,
    pub total: u64,
}

impl Download {
    pub async fn chunk(&mut self) -> Result<Option<Bytes>> {
        let begin = self.progress;
        let end = (begin + (self.total / 10)).min(self.total);

        if self.total <= self.progress {
            debug!("finished!");
            return Ok(None);
        }
        info!("fetching range {begin} => {end}");
        let mut headers = HeaderMap::new();
        headers.insert(
            RANGE,
            HeaderValue::from_str(&format!("bytes={begin}-{end}"))?,
        );
        let future = self.client.client.get(&self.url).headers(headers).send();
        let response = if let Some(timeout) = self.timeout {
            if let Ok(bytes) = time::timeout(timeout, future).await? {
                bytes
            } else {
                bail!("Download timed out due to inactivity");
            }
        } else {
            future.await?
        };

        // filter out EOF
        let bytes = response
            .bytes_stream()
            .collect::<Vec<_>>()
            .await
            .iter()
            .filter_map(|chunk| match chunk {
                Ok(bytes) => Some(bytes.to_vec()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .concat();
        self.progress += bytes.len() as u64;
        Ok(Some(bytes.into()))
    }
}
