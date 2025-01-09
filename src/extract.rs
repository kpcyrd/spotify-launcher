use crate::args::Args;
use crate::errors::*;
use crate::paths;
use libflate::gzip::Decoder;
use lzma::LzmaReader;
use std::io::Read;
use std::path::Path;
use tokio::fs;

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

pub async fn pkg<R: Read>(deb: R, args: &Args, install_path: &Path) -> Result<()> {
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
    bail!("Failed to find data entry in .deb");
}
