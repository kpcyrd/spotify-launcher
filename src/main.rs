use clap::Parser;
use env_logger::Env;
use spotify_launcher::apt;
use spotify_launcher::apt::Client;
use spotify_launcher::args::Args;
use spotify_launcher::config::ConfigFile;
use spotify_launcher::errors::*;
use spotify_launcher::extract;
use spotify_launcher::paths;
use spotify_launcher::ui;
use std::ffi::CString;
use std::path::Path;
use std::time::Duration;
use std::time::SystemTime;
use tokio::fs;

const UPDATE_CHECK_INTERVAL: u64 = 3600 * 24;
const KEYRING_DEFAULT_PATH: &str = "/usr/share/spotify-launcher/keyring.pgp";

struct VersionCheck {
    deb: Option<Vec<u8>>,
    version: String,
}

async fn should_update(args: &Args, cf: &ConfigFile, state: Option<&paths::State>) -> Result<bool> {
    if args.force_update || args.check_update || args.deb.is_some() || cf.launcher.check_update {
        Ok(true)
    } else if args.skip_update || cf.launcher.skip_update {
        Ok(false)
    } else if let Some(state) = &state {
        let Ok(since_update) = SystemTime::now().duration_since(state.last_update_check) else {
            // if the last update time is somehow in the future, check for updates now
            return Ok(true);
        };

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

async fn print_deb_url(args: &Args, keyring_path: &Path) -> Result<()> {
    let client = Client::new(args.timeout)?;
    let pkg = client.fetch_pkg_release(keyring_path).await?;
    println!("{}", pkg.download_url());
    Ok(())
}

async fn update(
    args: &Args,
    state: Option<&paths::State>,
    install_path: &Path,
    download_attempts: usize,
    keyring_path: &Path,
) -> Result<()> {
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
        let pkg = client.fetch_pkg_release(keyring_path).await?;

        match state {
            Some(state) if state.version == pkg.version && !args.force_update => {
                info!("Latest version is already installed, not updating");
                VersionCheck {
                    deb: None,
                    version: pkg.version,
                }
            }
            _ => {
                let deb = client.download_pkg(&pkg, download_attempts).await?;
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
        exec_args.push(CString::new(format!("--uri={}", uri))?);
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

    let download_attempts = args.download_attempts.unwrap_or_else(|| {
        cf.spotify
            .download_attempts
            .unwrap_or(apt::DEFAULT_DOWNLOAD_ATTEMPTS)
    });

    let keyring_path = args.keyring.clone().unwrap_or_else(|| {
        cf.launcher
            .keyring
            .clone()
            .unwrap_or(Path::new(KEYRING_DEFAULT_PATH).to_path_buf())
    });
    debug!("Using keyring path: {:?}", keyring_path);

    if args.print_deb_url {
        print_deb_url(&args, &keyring_path).await?;
    } else {
        let state = paths::load_state_file().await?;
        if should_update(&args, &cf, state.as_ref()).await? {
            if let Err(err) = update(
                &args,
                state.as_ref(),
                &install_path,
                download_attempts,
                &keyring_path,
            )
            .await
            {
                error!("Update failed: {err:#}");
                ui::error(&err).await?;
            }
        } else {
            info!("No update needed");
        }
        start(&args, &cf, &install_path)?;
    }

    Ok(())
}
