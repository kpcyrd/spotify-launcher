use crate::errors::*;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};

pub struct ProgressBar {
    child: Child,
}

impl ProgressBar {
    pub fn spawn() -> Result<ProgressBar> {
        let child = Command::new("zenity")
            .args([
                "--progress",
                "--title",
                "Downloading spotify",
                "--text=Downloading...",
                "--no-cancel",
                "--ok-label",
                "😺",
            ])
            .stdin(Stdio::piped())
            .spawn()
            .context("Failed to spawn zenity")?;
        Ok(ProgressBar { child })
    }

    pub async fn update(&mut self, progress: u64) -> Result<()> {
        if let Some(stdin) = &mut self.child.stdin {
            let buf = format!("{}\n", progress);
            stdin.write_all(buf.as_bytes()).await?;
            stdin.flush().await?;
        }
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.child.kill().await?;
        Ok(())
    }
}

pub async fn display_error(message: &str) -> Result<()> {
    let mut child = Command::new("zenity")
        .args(["--error", "--title", "spotify-launcher", "--text", message])
        .spawn()
        .context(format!("Failed to spawn zenity for error {}", message))?;
    child.wait().await?;
    Ok(())
}
