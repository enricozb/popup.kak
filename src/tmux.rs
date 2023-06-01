use std::{process::Command, time::SystemTime};

use anyhow::{Context, Result};
use tokio::process::Command as TokioCommand;

pub struct Tmux {
  session: String,
}

impl Tmux {
  pub fn new(command: &str, width: usize, height: usize) -> Result<Self> {
    let session = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)?
      .as_nanos()
      .to_string();

    let tmux = Self { session };
    tmux.start(command, width, height)?;

    Ok(tmux)
  }

  fn start(&self, command: &str, width: usize, height: usize) -> Result<()> {
    sync_command(
      "new-session",
      [
        "-s",
        &self.session,
        "-x",
        &width.to_string(),
        "-y",
        &height.to_string(),
        "-d",
      ],
    )?;

    self.sync_send_keys(command)?;

    Ok(())
  }

  pub fn sync_send_keys(&self, keys: &str) -> Result<()> {
    sync_command("send-keys", ["-t", &self.session, keys])?;

    Ok(())
  }

  fn kill(&self) -> Result<()> {
    sync_command("kill-session", ["-t", &self.session])?;

    Ok(())
  }

  pub async fn async_send_keys(&self, keys: &str) -> Result<()> {
    async_command("send-keys", ["-t", &self.session, keys]).await?;

    Ok(())
  }

  pub async fn capture_pane(&self) -> Result<Vec<u8>> {
    // TODO: add -e for escape sequences
    async_command("capture-pane", ["-t", &self.session, "-p"]).await
  }
}

impl Drop for Tmux {
  fn drop(&mut self) {
    self.kill().expect("kill");
  }
}

fn sync_command<const N: usize>(command: &str, args: [&str; N]) -> Result<Vec<u8>> {
  let output = Command::new("tmux")
    .arg(command)
    .args(args)
    .output()
    .with_context(|| format!("tmux {command}"))?;

  if !output.status.success() {
    return Err(anyhow::anyhow!(
      "tmux {command} exited with non-zero status: {}",
      output.status
    ));
  }
  Ok(output.stdout)
}

async fn async_command<const N: usize>(command: &str, args: [&str; N]) -> Result<Vec<u8>> {
  let output = TokioCommand::new("tmux")
    .arg(command)
    .args(args)
    .output()
    .await
    .with_context(|| format!("tmux {command}"))?;

  if !output.status.success() {
    return Err(anyhow::anyhow!(
      "tmux {command} exited with non-zero status: {}",
      output.status
    ));
  }
  Ok(output.stdout)
}
