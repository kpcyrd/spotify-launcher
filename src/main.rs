use clap::Parser;
use env_logger::Env;
use spotify_launcher::apt::Client;
use spotify_launcher::args::Args;
use spotify_launcher::config::ConfigFile;
use spotify_launcher::errors::*;
use spotify_launcher::extract;
use spotify_launcher::paths;
use std::ffi::CString;
use std::path::Path;
use std::time::Duration;
use std::time::SystemTime;
use tokio::fs;

const UPDATE_CHECK_INTERVAL: u64 = 3600 * 24;

struct VersionCheck {
    deb: Option<Vec<u8>>,
    version: String,
}

async fn should_update(args: &Args, state: Option<&paths::State>) -> Result<bool> {
    if args.force_update || args.check_update || args.deb.is_some() {
        Ok(true)
    } else if args.skip_update {
        Ok(false)
    } else if let Some(state) = &state {
        let since_update = SystemTime::now().duration_since(state.last_update_check)?;

        let hours_since = since_update.as_secs() / 3600;
        let days_since = hours_since / 24;
        let hours_since = hours_since % 24;

        debug!(
            "Last update check was {} days and {} hours ago",
            days_since, hours_since
        );
        Ok(since_update >= Duration::from_secs(UPDATE_CHECK_INTERVAL))
    } else {
        Ok(true)
    }
}

async fn print_deb_url(args: &Args) -> Result<()> {
    let client = Client::new(args.timeout)?;
    let pkg = client.fetch_pkg_release(&args.keyring).await?;
    println!("{}", pkg.download_url());
    Ok(())
}

async fn update(args: &Args, state: Option<&paths::State>, install_path: &Path) -> Result<()> {
    let update = if let Some(deb_path) = &args.deb {
        let deb = fs::read(deb_path)
            .await
            .with_context(|| anyhow!("Failed to read .deb file from {:?}", deb_path))?;
        VersionCheck {
            deb: Some(deb),
            version: "0".to_string(),
        }
    } else {
        let client = Client::new(args.timeout)?;
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
        extract::pkg(&deb[..], args, install_path).await?;
    }

    debug!("Updating state file");
    let buf = serde_json::to_string(&paths::State {
        last_update_check: SystemTime::now(),
        version: update.version,
    })?;
    fs::write(paths::state_file_path()?, buf)
        .await
        .context("Failed to write state file")?;

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
        exec_args.push(CString::new(uri.clone())?);
    }

    debug!("Assembled command: {:?}", exec_args);

    if args.no_exec {
        info!("Skipping exec because --no-exec was used");
    } else {
        cf.spotify.extra_env_vars.iter().for_each(|x| {
            let (k, v) = match x.split_once('=') {
                None => (x.as_str(), ""),
                Some(x) => x,
            };
            std::env::set_var(k, v);
        });
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

    if args.print_deb_url {
        print_deb_url(&args).await?;
    } else {
        let state = paths::load_state_file().await?;
        if should_update(&args, state.as_ref()).await? {
            update(&args, state.as_ref(), &install_path).await?;
        } else {
            info!("No update needed");
        }
        start(&args, &cf, &install_path)?;
    }

    Ok(())
}
