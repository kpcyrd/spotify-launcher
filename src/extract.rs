use crate::args::Args;
use crate::errors::*;
use crate::paths;
use libflate::gzip::Decoder;
use lzma::LzmaReader;
use std::io::Read;
use std::path::Path;
use tokio::fs;

#[allow(unused_imports)] // bring the Parser trait into scope, so we can use "Args::parse_from"
use clap::Parser;

const ERROR_NO_DATA_IN_DEB: &str = "No data.tar.gz or data.tar.xz found in .deb";

async fn atomic_swap(src: &Path, target: &Path) -> Result<()> {
    info!(
        "Atomically swapping new directory at {:?} with {:?}...",
        src, target
    );
    fs::create_dir(target).await.ok();
    libxch::xch(src, target)?;
    Ok(())
}

async fn extract_data<R: Read>(
    mut tar: tar::Archive<R>,
    args: &Args,
    install_path: &Path,
) -> Result<()> {
    let new_install_path = if let Some(path) = args.install_dir.clone() {
        path
    } else {
        paths::new_install_path()?
    };

    info!("Extracting to {:?}...", new_install_path);
    tar.unpack(&new_install_path)
        .context("Failed to extract spotify")?;

    if install_path != new_install_path {
        if let Err(err) = atomic_swap(&new_install_path, install_path).await {
            warn!("Failed to swap {new_install_path:?} with {install_path:?}: {err:#}");
            debug!("Falling back to non-atomic swap, removing old directory...");
            fs::remove_dir_all(&install_path)
                .await
                .context("Failed to delete old directory")?;
            debug!("Moving new directory in place...");
            fs::rename(&new_install_path, &install_path)
                .await
                .context("Failed to move new directory in place")?;
        } else {
            debug!("Removing old directory...");
            if let Err(err) = fs::remove_dir_all(&new_install_path).await {
                warn!("Failed to delete old directory: {:#}", err);
            }
        }
    }
    Ok(())
}

pub async fn pkg<R: Read>(deb: R, args: &Args, install_path: &Path) -> Result<(), Error> {
    let mut ar = ar::Archive::new(deb);
    while let Some(entry) = ar.next_entry() {
        let mut entry = entry?;
        match entry.header().identifier() {
            b"data.tar.gz" => {
                debug!("Found data.tar.gz in .deb");
                let decoder = Decoder::new(&mut entry)?;
                let tar = tar::Archive::new(decoder);
                return extract_data(tar, args, install_path).await;
            }
            b"data.tar.xz" => {
                debug!("Found data.tar.xz in .deb");
                let decoder = LzmaReader::new_decompressor(entry)?;
                let tar = tar::Archive::new(decoder);
                return extract_data(tar, args, install_path).await;
            }
            _ => (),
        }
    }
    return Err(anyhow!(ERROR_NO_DATA_IN_DEB));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    // Make sure that parsing a .deb is ok
    async fn test_verify_deb_error() -> Result<(), Error> {
        let deb_path = "data/tests/fake_deb_for_unit_testing.none.deb";

        let args: Args = Args::parse_from(&["app_name"]);
        let install_path = dirs::runtime_dir()
            .map(|path| path.join("data/tests/install"))
            .expect("Could not get the runtime directory");

        tokio::fs::create_dir_all(install_path.clone()).await?;

        let deb = tokio::fs::read(deb_path)
            .await
            .with_context(|| anyhow!("Failed to read .deb file from {:?}", deb_path))?;

        let result = crate::extract::pkg(&deb[..], &args, &install_path).await;

        if result.is_err() {
            assert_eq!(ERROR_NO_DATA_IN_DEB, result.err().unwrap().to_string());
            return Ok(());
        }

        return Err(anyhow!("Error"));
    }
}
