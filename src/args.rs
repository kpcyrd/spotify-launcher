use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[command(version)]
pub struct Args {
    /// Overwrite the default keyring
    #[arg(long)]
    pub keyring: Option<PathBuf>,
    /// Use a local .deb file instead of downloading one
    #[arg(long)]
    pub deb: Option<PathBuf>,
    /// Install into specific directory
    #[arg(long)]
    pub install_dir: Option<PathBuf>,
    /// Verbose logs (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// Uri to pass to the spotify child process
    pub uri: Option<String>,
    /// Always check for updates when starting
    #[arg(long)]
    pub check_update: bool,
    /// Never check for updates when starting
    #[arg(long)]
    pub skip_update: bool,
    /// Update even if latest version is already installed
    #[arg(long)]
    pub force_update: bool,
    /// Check for the latest .deb and print its url
    #[arg(long)]
    pub print_deb_url: bool,
    /// Run the install/update code but don't actually run the final binary
    #[arg(long)]
    pub no_exec: bool,
    /// The timeout to use for http connections and requests
    #[arg(long)]
    pub timeout: Option<u64>,
    /// How often to try to resume the download until giving up (0 for unlimited)
    #[arg(long)]
    pub download_attempts: Option<usize>,
}
