use crate::errors::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub spotify: SpotifyConfig,
    #[serde(default)]
    pub launcher: LauncherConfig,
}

impl ConfigFile {
    pub fn parse(s: &str) -> Result<ConfigFile> {
        let mut c = toml::from_str::<ConfigFile>(s)?;

        if c.launcher.force_update {
            c.launcher.skip_update = false;
            c.launcher.check_update = true;
        } else {
            c.launcher.check_update = !c.launcher.skip_update;
        }

        if let Some(keyring) = &c.launcher.keyring {
            if !keyring.exists() {
                c.launcher.keyring = None;
            }
        }

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

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            spotify: SpotifyConfig::default(),
            launcher: LauncherConfig {
                check_update: true,
                ..LauncherConfig::default()
            },
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

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LauncherConfig {
    #[serde(default)]
    pub skip_update: bool,
    #[serde(default)]
    pub check_update: bool,
    #[serde(default)]
    pub keyring: Option<PathBuf>,
    #[serde(default)]
    force_update: bool,
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

    #[test]
    fn test_empty_launcher_config() -> Result<()> {
        let cf = ConfigFile::parse("[launcher]")?;
        assert_eq!(cf, ConfigFile::default());
        Ok(())
    }

    #[test]
    fn test_launcher_config_skip_update() -> Result<()> {
        let cf = ConfigFile::parse(
            r#"[launcher]
skip_update = true
        "#,
        );
        assert_eq!(
            cf.unwrap_or_default(),
            ConfigFile {
                spotify: SpotifyConfig::default(),
                launcher: LauncherConfig {
                    skip_update: true,
                    check_update: false,
                    force_update: false,
                    keyring: None,
                }
            }
        );

        Ok(())
    }

    #[test]
    fn test_launcher_config_skip_update_check_update() -> Result<()> {
        let cf = ConfigFile::parse(
            r#"[launcher]
skip_update = true
check_update = true
        "#,
        );
        assert_eq!(
            cf.unwrap_or_default(),
            ConfigFile {
                spotify: SpotifyConfig::default(),
                launcher: LauncherConfig {
                    skip_update: true,
                    check_update: false,
                    force_update: false,
                    keyring: None,
                }
            }
        );

        Ok(())
    }

    #[test]
    fn test_launcher_config_skip_update_force_update() -> Result<()> {
        let cf = ConfigFile::parse(
            r#"[launcher]
skip_update = true
force_update = true
        "#,
        );
        assert_eq!(
            cf.unwrap_or_default(),
            ConfigFile {
                spotify: SpotifyConfig::default(),
                launcher: LauncherConfig {
                    skip_update: false,
                    check_update: true,
                    force_update: true,
                    keyring: None,
                }
            }
        );

        Ok(())
    }

    #[test]
    fn test_launcher_config_check_update_force_update_skip_update() -> Result<()> {
        let cf = ConfigFile::parse(
            r#"[launcher]
skip_update = true
force_update = true
        "#,
        );
        assert_eq!(
            cf.unwrap_or_default(),
            ConfigFile {
                spotify: SpotifyConfig::default(),
                launcher: LauncherConfig {
                    skip_update: false,
                    check_update: true,
                    force_update: true,
                    keyring: None,
                }
            }
        );

        Ok(())
    }

    #[test]
    fn test_launcher_config_keyring() -> Result<()> {
        let cf = ConfigFile::parse(
            r#"[launcher]
keyring = "/dev/null"
        "#,
        );
        assert_eq!(
            cf.unwrap_or_default(),
            ConfigFile {
                spotify: SpotifyConfig::default(),
                launcher: LauncherConfig {
                    skip_update: false,
                    check_update: true,
                    force_update: false,
                    keyring: Some(PathBuf::from("/dev/null")),
                }
            }
        );

        Ok(())
    }

    #[test]
    fn test_launcher_config_keyring_empty_string() -> Result<()> {
        let cf = ConfigFile::parse(
            r#"[launcher]
keyring = ""
        "#,
        );
        assert_eq!(
            cf.unwrap_or_default(),
            ConfigFile {
                spotify: SpotifyConfig::default(),
                launcher: LauncherConfig {
                    skip_update: false,
                    check_update: true,
                    force_update: false,
                    keyring: None,
                }
            }
        );

        Ok(())
    }
}
