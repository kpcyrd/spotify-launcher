use crate::errors::*;
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
            .timeout(Duration::from_secs(3600)) // kinda arbitrary
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
}
