use crate::errors::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub spotify: SpotifyConfig,
}

impl ConfigFile {
    pub fn parse(s: &str) -> Result<ConfigFile> {
        let c = toml::from_str(s)?;
        Ok(c)
    }

    pub fn load_from(path: &Path) -> Result<ConfigFile> {
        info!("Loading configuration file at {:?}", path);
        let buf = fs::read_to_string(path)
            .with_context(|| anyhow!("Failed to read config file at {:?}", path))?;
        Self::parse(&buf)
    }

    pub fn locate_file() -> Result<Option<PathBuf>> {
        for path in [dirs::config_dir(), Some(PathBuf::from("/etc/"))]
            .into_iter()
            .flatten()
        {
            let path = path.join("spotify-launcher.conf");
            debug!("Searching for configuration file at {:?}", path);
            if path.exists() {
                debug!("Found configuration file at {:?}", path);
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    pub fn load() -> Result<ConfigFile> {
        if let Some(path) = Self::locate_file()? {
            Self::load_from(&path)
        } else {
            info!("No configuration file found, using default config");
            Ok(Self::default())
        }
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpotifyConfig {
    #[serde(default)]
    pub extra_arguments: Vec<String>,
    #[serde(default)]
    pub extra_env_vars: Vec<String>,
    pub download_attempts: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_config() -> Result<()> {
        let cf = ConfigFile::parse("")?;
        assert_eq!(cf, ConfigFile::default());
        Ok(())
    }

    #[test]
    fn test_empty_spotify_config() -> Result<()> {
        let cf = ConfigFile::parse("[spotify]")?;
        assert_eq!(cf, ConfigFile::default());
        Ok(())
    }
}
