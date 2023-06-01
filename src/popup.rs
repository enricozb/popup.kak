use std::{process::Command, time::SystemTime};

use anyhow::{Context, Result};

use crate::Args;

pub struct Popup {
  command: String,
  command_fifo: String,
  keys_fifo: String,
  height: usize,
  width: usize,

  pub tmux_session: String,
}

impl Popup {
  pub fn new(args: Args) -> Result<Self> {
    Ok(Self {
      command: args.command,
      command_fifo: args.command_fifo,
      keys_fifo: args.keys_fifo,
      height: args.height.checked_sub(5).ok_or(anyhow::anyhow!("height too small"))?,
      width: args.width.checked_sub(5).ok_or(anyhow::anyhow!("width too small"))?,
      tmux_session: Self::new_session()?,
    })
  }

  fn new_session() -> Result<String> {
    let duration_since_epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let session_name = duration_since_epoch.as_nanos().to_string();

    let status = Command::new("tmux")
      .args(["new-session", "-d", "-s", &session_name])
      .status()
      .context("tmux")?;

    if !status.success() {
      return Err(anyhow::anyhow!("tmux exited with non-zero status: {status}"));
    }

    Ok(session_name)
  }

  pub fn start(&self) -> Result<()> {
    Ok(())
  }
}

impl Drop for Popup {
  fn drop(&mut self) {
    let status = Command::new("tmux")
      .args(["kill-session", "-t", &self.tmux_session])
      .status()
      .expect("tmux kill-session");

    assert!(
      status.success(),
      "tmux kill-session exited with non-zero status: {status}"
    );
  }
}
