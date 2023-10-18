use std::{process::Command, time::SystemTime};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::geometry::{Point, Size};

#[derive(Deserialize)]
pub struct DisplayInfo {
  pub size: Size,
  pub cursor: Point,
}

#[derive(Clone)]
pub struct Tmux {
  pub session: String,
}

impl Tmux {
  pub fn new(command: &str, size: Size) -> Result<Self> {
    let session = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)?
      .as_nanos()
      .to_string();

    let tmux = Self { session };
    tmux.start(command, size)?;
    tmux.set_option("status", "off")?;

    // if an attached tmux session is running, tmux does not respect
    // the `-x`/`-y` size options when creating a new session.
    // NOTE: this could also be done by having a `size` member to this
    // struct, and then comparing it to what we get back during `Self::display`.
    tmux.resize_window(size)?;

    Ok(tmux)
  }

  fn start(&self, command: &str, size: Size) -> Result<()> {
    tmux_command(
      "new-session",
      [
        "-s",
        &self.session,
        "-x",
        &size.width.to_string(),
        "-y",
        &size.height.to_string(),
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
    tmux_command("capture-pane", ["-t", &self.session, "-p", "-e"])
  }

  pub fn display_info(&self) -> Result<DisplayInfo> {
    const FORMAT_STR: &str = r#"{
      "size": {
        "width": #{pane_width},
        "height": #{pane_height}
      },
      "cursor": {
        "x": #{cursor_x},
        "y": #{cursor_y}
      }
    }"#;

    // OpenSUSE's tmux replaces newlines with _ so we remove the newlines
    let format_str = FORMAT_STR.replace('\n', " ");
    let content = tmux_command("display", ["-t", &self.session, "-p", &format_str])?;

    serde_json::from_slice(&content).with_context(|| format!("Failed to parse: {}", String::from_utf8_lossy(&content)))
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
      "tmux {command} exited with non-zero status: {}, err: {}",
      output.status,
      String::from_utf8_lossy(&output.stderr),
    ));
  }

  Ok(output.stdout)
}
