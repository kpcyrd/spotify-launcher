use crate::errors::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

pub fn spotify_launcher_path() -> Result<PathBuf> {
    let path = dirs::data_dir().context("Failed to detect data directory")?;
    Ok(path.join("spotify-launcher"))
}

pub fn install_path() -> Result<PathBuf> {
    let path = spotify_launcher_path()?;
    Ok(path.join("install"))
}

pub fn new_install_path() -> Result<PathBuf> {
    let path = spotify_launcher_path()?;
    Ok(path.join("install-new"))
}

pub fn state_file_path() -> Result<PathBuf> {
    let path = spotify_launcher_path()?;
    Ok(path.join("state.json"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub version: String,
    pub last_update_check: SystemTime,
}

pub fn load_state_file() -> Result<Option<State>> {
    let state_file_path = state_file_path()?;
    if state_file_path.exists() {
        debug!("Reading state file from {:?}...", state_file_path);
        let buf =
            fs::read(&state_file_path).context("Failed to read spotify-launcher state file")?;
        let state = serde_json::from_slice::<State>(&buf);
        debug!("Loaded state: {:?}", state);
        Ok(state.ok())
    } else {
        debug!("State file does not exist, using empty state");
        Ok(None)
    }
}
