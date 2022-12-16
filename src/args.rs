use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
pub struct Args {
    /// Overwrite the default keyring
    #[clap(long, default_value = "/usr/share/spotify-launcher/keyring.pgp")]
    pub keyring: PathBuf,
    /// Use a local .deb file instead of downloading one
    #[clap(long)]
    pub deb: Option<PathBuf>,
    /// Install into specific directory
    #[clap(long)]
    pub install_dir: Option<PathBuf>,
    /// Verbose logs (can be used multiple times)
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// Uri to pass to the spotify child process
    pub uri: Option<String>,
    /// Always check for updates when starting
    #[clap(long)]
    pub check_update: bool,
    /// Always check for updates when starting
    #[clap(long)]
    pub skip_update: bool,
    /// Update even if latest version is already installed
    #[clap(long)]
    pub force_update: bool,
    /// Run the install/update code but don't actually run the final binary
    #[clap(long)]
    pub no_exec: bool,
}
