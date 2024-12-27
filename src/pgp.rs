use crate::errors::*;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

pub async fn verify_sig<P: AsRef<Path>>(sig: P, artifact: P, keyring: P) -> Result<()> {
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

    if exit.success() {
        Ok(())
    } else {
        bail!("Verification of pgp signature didn't succeed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verify() -> Result<()> {
        verify_sig(
            "data/Release.gpg",
            "data/Release",
            "data/pubkey_C85668DF69375001.gpg",
        )
        .await
    }
}
