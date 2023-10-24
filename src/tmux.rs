use std::{ffi::OsStr, process::Command, sync::Arc, time::SystemTime};

use anyhow::{Context, Result};
use parking_lot::Mutex;
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

  size: Arc<Mutex<Size>>,
}

impl Tmux {
  pub fn new(command: &[String], size: Size) -> Result<Self> {
    let session = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)?
      .as_nanos()
      .to_string();

    let tmux = Self {
      session,
      size: Arc::new(Mutex::new(size)),
    };

    tmux.start(command, size)?;
    tmux.set_option("status", "off")?;

    Ok(tmux)
  }

  fn start(&self, command: &[String], size: Size) -> Result<()> {
    let width = size.width.to_string();
    let height = size.height.to_string();

    let mut args = vec![
      ";",
      "new-session",
      "-s",
      &self.session,
      "-x",
      &width,
      "-y",
      &height,
      "-d",
      "--",
    ];

    args.extend(command.iter().map(String::as_str));

    tmux_command("start", &args)?;

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

    let display_info: DisplayInfo = serde_json::from_slice(&content)
      .with_context(|| format!("Failed to parse: {}", String::from_utf8_lossy(&content)))?;

    let current_size = *self.size.lock();

    if display_info.size != current_size {
      self.resize_window(current_size)?;
    }

    Ok(display_info)
  }

  pub fn set_size(&self, size: Size) -> Result<()> {
    *self.size.lock() = size;

    self.resize_window(size)
  }

  fn resize_window(&self, size: Size) -> Result<()> {
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

fn tmux_command<I, S>(command: &str, args: I) -> Result<Vec<u8>>
where
  I: IntoIterator<Item = S>,
  S: AsRef<OsStr>,
{
  let output = Command::new("tmux")
    .args(["-L", "kak-popup"])
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
