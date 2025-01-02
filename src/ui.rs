use crate::errors::*;
use std::process::Stdio;
use tokio::process::{Child, Command};

pub struct Zenity {
    pub child: Child,
}

impl Zenity {
    pub fn spawn(args: &[&str]) -> Result<Self> {
        let child = Command::new("zenity")
            .args(args)
            .stdin(Stdio::piped())
            .spawn()
            .context("Failed to spawn zenity")?;
        Ok(Self { child })
    }
}

fn format_error(err: &Error) -> String {
    let mut chain = err.chain().peekable();
    let Some(err) = chain.next() else {
        return "An unknown error has occured".to_string();
    };

    let mut msg = format!("{err}");
    if chain.peek().is_some() {
        msg.push_str("\n\nCaused by:");
        for err in chain {
            msg.push_str(&format!("\n • {err}"));
        }
    }

    msg
}

pub async fn error(err: &Error) -> Result<()> {
    let msg = format_error(err);
    let mut ui = Zenity::spawn(&["--error", "--no-markup", "--text", &msg])?;
    ui.child.wait().await?;
    Ok(())
}
