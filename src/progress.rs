use crate::errors::*;
use crate::ui::Zenity;
use tokio::io::AsyncWriteExt;

pub struct ProgressBar {
    ui: Zenity,
}

impl ProgressBar {
    pub fn spawn() -> Result<ProgressBar> {
        let ui = Zenity::spawn(&[
            "--progress",
            "--title",
            "Downloading spotify",
            "--text=Downloading...",
            "--no-cancel",
            "--ok-label",
            "😺",
        ])?;
        Ok(ProgressBar { ui })
    }

    pub async fn update(&mut self, progress: u64) -> Result<()> {
        if let Some(stdin) = &mut self.ui.child.stdin {
            let buf = format!("{}\n", progress);
            stdin.write_all(buf.as_bytes()).await?;
            stdin.flush().await?;
        }
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.ui.child.kill().await?;
        Ok(())
    }
}
