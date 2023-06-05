use std::{process::Command, time::SystemTime};

use anyhow::{Context, Result};

use crate::geometry::Size;

#[derive(Clone)]
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
    tmux_command(
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
    tmux_command("kill-session", ["-t", &self.session])?;

    Ok(())
  }

  fn set_option(&self, option: &str, value: &str) -> Result<()> {
    tmux_command("set-option", ["-t", &self.session, option, value])?;

    Ok(())
  }

  pub fn send_keys(&self, keys: &str) -> Result<()> {
    tmux_command("send-keys", ["-t", &self.session, keys])?;

    Ok(())
  }

  pub fn capture_pane(&self) -> Result<Vec<u8>> {
    // TODO: add -e for escape sequences
    tmux_command("capture-pane", ["-t", &self.session, "-p"])
  }

  pub fn resize_window(&self, size: Size) -> Result<()> {
    tmux_command(
      "resize-window",
      [
        "-t",
        &self.session,
        "-x",
        &size.width.to_string(),
        "-y",
        &size.height.to_string(),
      ],
    )?;

    Ok(())
  }
}

fn tmux_command<const N: usize>(command: &str, args: [&str; N]) -> Result<Vec<u8>> {
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
