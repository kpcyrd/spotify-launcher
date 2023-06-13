use crate::crypto;
use crate::deb::{self, Pkg};
use crate::errors::*;
use crate::http;
use crate::pgp;
use crate::progress::ProgressBar;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub struct Client {
    client: http::Client,
}

impl Client {
    pub fn new(timeout: Option<u64>) -> Result<Client> {
        let client = http::Client::new(timeout)?;
        Ok(Client { client })
    }

    pub async fn fetch_pkg_release(&self, keyring_path: &Path) -> Result<Pkg> {
        info!("Downloading release file...");
        let release = self
            .client
            .fetch("http://repository.spotify.com/dists/testing/Release")
            .await?;

        info!("Downloading signature...");
        let sig = self
            .client
            .fetch("http://repository.spotify.com/dists/testing/Release.gpg")
            .await?;

        info!("Verifying pgp signature...");
        let tmp = tempfile::tempdir().context("Failed to create temporary directory")?;
        let tmp_path = tmp.path();

        let artifact_path = tmp_path.join("artifact");
        fs::write(&artifact_path, &release)?;
        let sig_path = tmp_path.join("sig");
        fs::write(&sig_path, &sig)?;

        pgp::verify_sig::<&Path>(&sig_path, &artifact_path, keyring_path).await?;

        info!("Signature verified successfully!");
        let release = deb::parse_release_file(&String::from_utf8(release)?)?;
        let arch = deb::Architecture::current();
        let debian_arch_str = arch.to_debian_str();

        if !release.architectures.iter().any(|a| a == debian_arch_str) {
            bail!(
                "There are no packages for your cpu's architecture (cpu={:?}, supported={:?})",
                debian_arch_str,
                release.architectures
            )
        }

        let packages_path = format!("non-free/binary-{debian_arch_str}/Packages");

        let packages_sha256sum = release
            .sha256_sums
            .get(&packages_path)
            .context("Missing sha256sum for package index")?;

        info!("Downloading package index...");
        let pkg_index = self
            .client
            .fetch(&format!("http://repository.spotify.com/dists/testing/{packages_path}"))
            .await?;

        info!("Verifying with sha256sum hash...");
        let downloaded_sha256sum = crypto::sha256sum(&pkg_index);
        if *packages_sha256sum != downloaded_sha256sum {
            bail!(
                "Downloaded bytes don't match signed sha256sum (signed: {:?}, downloaded: {:?})",
                packages_sha256sum,
                downloaded_sha256sum
            );
        }

        let pkg_index = deb::parse_package_index(&String::from_utf8(pkg_index)?)?;
        debug!("Parsed package index: {:?}", pkg_index);
        let pkg = pkg_index
            .into_iter()
            .find(|p| p.package == "spotify-client")
            .context("Repository didn't contain spotify-client")?;

        debug!("Found package: {:?}", pkg);
        Ok(pkg)
    }

    pub async fn download_pkg(&self, pkg: &Pkg) -> Result<Vec<u8>> {
        let filename = pkg
            .filename
            .rsplit_once('/')
            .map(|(_, x)| x)
            .unwrap_or("???");

        info!(
            "Downloading deb file for {:?} version={:?} ({:?})",
            filename, pkg.package, pkg.version
        );
        let url = pkg.download_url();

        // download
        let mut pb = ProgressBar::spawn()?;
        let mut dl = self.client.fetch_stream(&url).await?;
        let mut deb = Vec::new();
        let mut hasher = Sha256::new();
        while let Some(chunk) = dl.chunk().await? {
            deb.extend(&chunk);
            hasher.update(&chunk);
            let progress = (dl.progress as f64 / dl.total as f64 * 100.0) as u64;
            pb.update(progress).await?;
            debug!(
                "Download progress: {}%, {}/{}",
                progress, dl.progress, dl.total
            );
        }
        pb.close().await?;

        // verify checksum
        info!("Verifying with sha256sum hash...");
        let downloaded_sha256sum = format!("{:x}", hasher.finalize());
        if pkg.sha256sum != downloaded_sha256sum {
            bail!(
                "Downloaded bytes don't match signed sha256sum (signed: {:?}, downloaded: {:?})",
                pkg.sha256sum,
                downloaded_sha256sum
            );
        }

        Ok(deb)
    }
}
