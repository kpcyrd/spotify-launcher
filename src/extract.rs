use crate::args::Args;
use crate::errors::*;
use crate::paths;
use libflate::gzip::Decoder;
use std::io::Read;
use std::path::Path;
use tokio::fs;

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
        info!("Setting new directory active");
        fs::create_dir(install_path).await.ok();
        libxch::xch_non_atomic(install_path, &new_install_path).with_context(|| {
            anyhow!(
                "Failed to update directories {:?} and {:?}",
                install_path,
                new_install_path
            )
        })?;
        debug!("Removing old directory...");
        if let Err(err) = fs::remove_dir_all(&new_install_path).await {
            warn!("Failed to delete old directory: {:?}", err);
        }
    }
    Ok(())
}

pub async fn pkg<R: Read>(deb: R, args: &Args, install_path: &Path) -> Result<()> {
    let mut ar = ar::Archive::new(deb);
    while let Some(entry) = ar.next_entry() {
        let mut entry = entry?;

        if entry.header().identifier() == b"data.tar.gz" {
            debug!("Found data.tar.gz in .deb");
            let decoder = Decoder::new(&mut entry)?;
            let tar = tar::Archive::new(decoder);
            return extract_data(tar, args, install_path).await;
        }
    }
    bail!("Failed to find data entry in .deb");
}
