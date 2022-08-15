use clap::Parser;
use env_logger::Env;
use libflate::gzip::Decoder;
use spotify_launcher::apt::Client;
use spotify_launcher::args::Args;
use spotify_launcher::config::ConfigFile;
use spotify_launcher::errors::*;
use spotify_launcher::paths;
use std::ffi::CString;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::Duration;
use std::time::SystemTime;

const UPDATE_CHECK_INTERVAL: u64 = 3600 * 24;

fn extract_tar(mut deb: &[u8]) -> Result<Vec<u8>> {
    let mut ar_reader = ar::Archive::new(&mut deb);
    while let Some(entry) = ar_reader.next_entry() {
        let mut entry = entry?;

        if entry.header().identifier() == b"data.tar.gz" {
            debug!("Found data.tar.gz, decompressing...");
            let mut decoder = Decoder::new(&mut entry)?;
            let mut tar = Vec::new();
            decoder.read_to_end(&mut tar)?;

            debug!("Extracted tar data, {} bytes", tar.len());
            return Ok(tar);
        }
    }

    bail!("Failed to find data entry in .deb");
}

struct VersionCheck {
    deb: Option<Vec<u8>>,
    version: String,
}

async fn update(args: &Args, install_path: &Path) -> Result<()> {
    let state = paths::load_state_file()?;
    let should_update = if args.force_update || args.check_update {
        true
    } else if args.skip_update {
        false
    } else if let Some(state) = &state {
        let since_update = SystemTime::now().duration_since(state.last_update_check)?;

        let hours_since = since_update.as_secs() / 3600;
        let days_since = hours_since / 24;
        let hours_since = hours_since % 24;

        debug!(
            "Last update check was {} days and {} hours ago",
            days_since, hours_since
        );
        since_update >= Duration::from_secs(UPDATE_CHECK_INTERVAL)
    } else {
        true
    };

    if should_update {
        let update = if let Some(deb_path) = &args.deb {
            let deb = fs::read(&deb_path)
                .with_context(|| anyhow!("Failed to read .deb file from {:?}", deb_path))?;
            VersionCheck {
                deb: Some(deb),
                version: "0".to_string(),
            }
        } else {
            let client = Client::new()?;
            let pkg = client.fetch_pkg_release(&args.keyring).await?;

            match state {
                Some(state) if state.version == pkg.version && !args.force_update => {
                    info!("Latest version is already installed, not updating");
                    VersionCheck {
                        deb: None,
                        version: pkg.version,
                    }
                }
                _ => {
                    let deb = client.download_pkg(&pkg).await?;
                    VersionCheck {
                        deb: Some(deb),
                        version: pkg.version,
                    }
                }
            }
        };

        if let Some(deb) = update.deb {
            let tar = extract_tar(&deb).context("Failed to process .deb file")?;

            let mut tar = &tar[..];
            let mut tar = tar::Archive::new(&mut tar);

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
                fs::create_dir(&install_path).ok();
                libxch::xch_non_atomic(&install_path, &new_install_path).with_context(|| {
                    anyhow!(
                        "Failed to update directories {:?} and {:?}",
                        install_path,
                        new_install_path
                    )
                })?;
                debug!("Removing old directory...");
                if let Err(err) = fs::remove_dir_all(&new_install_path) {
                    warn!("Failed to delete old directory: {:?}", err);
                }
            }
        }

        debug!("Updating state file");
        let buf = serde_json::to_string(&paths::State {
            last_update_check: SystemTime::now(),
            version: update.version,
        })?;
        fs::write(&paths::state_file_path()?, buf).context("Failed to write state file")?;
    } else {
        info!("No update needed");
    }
    Ok(())
}

fn start(args: &Args, cf: &ConfigFile, install_path: &Path) -> Result<()> {
    let bin = install_path.join("usr/bin/spotify");
    let bin = CString::new(bin.to_string_lossy().as_bytes())?;

    let mut exec_args = vec![CString::new("spotify")?];

    for arg in cf.spotify.extra_arguments.iter().cloned() {
        exec_args.push(CString::new(arg)?);
    }

    if let Some(uri) = &args.uri {
        exec_args.push(CString::new(format!("--uri={}", uri))?);
    }

    debug!("Assembled command: {:?}", exec_args);

    if args.no_exec {
        info!("Skipping exec because --no-exec was used");
    } else {
        nix::unistd::execv(&bin, &exec_args)
            .with_context(|| anyhow!("Failed to exec {:?}", bin))?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let log_level = match args.verbose {
        0 => "info",
        1 => "info,spotify_launcher=debug",
        2 => "debug",
        _ => "trace",
    };
    env_logger::init_from_env(Env::default().default_filter_or(log_level));

    let cf = ConfigFile::load().context("Failed to load configuration")?;

    let install_path = if let Some(path) = &args.install_dir {
        path.clone()
    } else {
        paths::install_path()?
    };
    debug!("Using install path: {:?}", install_path);

    update(&args, &install_path).await?;
    start(&args, &cf, &install_path)?;

    Ok(())
}
