use crate::args::Args;
use crate::errors::*;
use clap::Parser;
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::fs;
use tokio::process::Command;

pub async fn verify_sig<P: AsRef<Path>>(
    sig: P,
    artifact: P,
    keyring: P,
) -> Result<(), anyhow::Error> {
    let mut cmd = Command::new("sqv")
        .arg("--keyring")
        .arg(keyring.as_ref())
        .arg("--")
        .arg(sig.as_ref())
        .arg(artifact.as_ref())
        .stdout(Stdio::null())
        .spawn()
        .context("Failed to run `sqv`")?;

    let exit = cmd
        .wait()
        .await
        .context("Failed to wait for `sqv` child process")?;

    exit.success()
        .then(|| Ok(()))
        .unwrap_or_else(|| Err(anyhow!("Verification of pgp signature didn't succeed")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verify() -> Result<()> {
        verify_sig(
            "data/Release.gpg",
            "data/Release",
            "data/pubkey_6224F9941A8AA6D1.gpg",
        )
        .await
    }
}
