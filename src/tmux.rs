use std::{process::Command, time::SystemTime};

use anyhow::{Context, Result};
use tokio::process::Command as TokioCommand;

pub struct Tmux {
  pub session: String,
}

impl Tmux {
  pub fn new(command: &str, height: usize, width: usize) -> Result<Self> {
    let session = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)?
      .as_nanos()
      .to_string();

    let tmux = Self { session };
    tmux.start(command, height, width)?;
    tmux.set_option("status", "off")?;

    Ok(tmux)
  }

  fn start(&self, command: &str, height: usize, width: usize) -> Result<()> {
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
        command,
      ],
    )?;

    Ok(())
  }

  pub fn kill(&self) -> Result<()> {
    sync_command("kill-session", ["-t", &self.session])?;

    Ok(())
  }

  fn set_option(&self, option: &str, value: &str) -> Result<()> {
    sync_command("set-option", ["-t", &self.session, option, value])?;

    Ok(())
  }

  pub async fn send_keys(&self, keys: &str) -> Result<()> {
    async_command("send-keys", ["-t", &self.session, keys]).await?;

    Ok(())
  }

  pub async fn capture_pane(&self) -> Result<Vec<u8>> {
    // TODO: add -e for escape sequences
    async_command("capture-pane", ["-t", &self.session, "-p"]).await
  }

  pub async fn resize_window(&self, height: usize, width: usize) -> Result<()> {
    async_command(
      "resize-window",
      ["-t", &self.session, "-x", &width.to_string(), "-y", &height.to_string()],
    )
    .await?;

    Ok(())
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
