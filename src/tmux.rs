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

pub enum Key {
  Key(String),
  Esc(String),
  /// Scroll the pane's scrollback by the given amount (positive is up).
  Scroll(i32),
}

#[derive(Clone)]
pub struct Tmux {
  pub session: String,

  size: Arc<Mutex<Size>>,
  scroll: Arc<Mutex<usize>>,
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
      scroll: Arc::new(Mutex::new(0)),
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

  pub fn send_keys(&self, keys: Key) -> Result<()> {
    match keys {
      Key::Key(s) => {
        *self.scroll.lock() = 0;
        tmux_command("send-keys", ["-t", &self.session, &s])?;
      }
      Key::Esc(s) => {
        *self.scroll.lock() = 0;
        tmux_command("send-keys", ["-t", &self.session, "-l", &format!("\x1b[{s}")])?;
      }
      Key::Scroll(amount) => self.scroll(amount)?,
    };

    Ok(())
  }

  /// Adjusts the scrollback offset by `amount` (positive is down), clamped to
  /// the pane's history size.
  ///
  /// If the pane is on the alternate screen (e.g. `less`, `man`) there is no
  /// scrollback, so arrow keys are sent instead, matching tmux's native
  /// mouse-wheel behavior.
  fn scroll(&self, amount: i32) -> Result<()> {
    if self.display_flag("alternate_on")? {
      let key = if amount > 0 { "Down" } else { "Up" };

      for _ in 0..amount.unsigned_abs() {
        tmux_command("send-keys", ["-t", &self.session, key])?;
      }

      return Ok(());
    }

    let mut scroll = self.scroll.lock();

    if amount > 0 {
      *scroll = scroll.saturating_sub(amount as usize);
    } else {
      *scroll = (*scroll + amount.unsigned_abs() as usize).min(self.history_size()?);
    }

    Ok(())
  }

  fn history_size(&self) -> Result<usize> {
    let output = tmux_command("display", ["-t", &self.session, "-p", "#{history_size}"])?;

    Ok(String::from_utf8_lossy(&output).trim().parse()?)
  }

  fn display_flag(&self, flag: &str) -> Result<bool> {
    let output = tmux_command("display", ["-t", &self.session, "-p", &format!("#{{{flag}}}")])?;

    Ok(output.starts_with(b"1"))
  }

  /// Whether the program running in the pane has enabled mouse tracking.
  pub fn mouse_support(&self) -> Result<bool> {
    self.display_flag("mouse_any_flag")
  }

  pub fn capture_pane(&self) -> Result<Vec<u8>> {
    let scroll = *self.scroll.lock();

    if scroll == 0 {
      return tmux_command("capture-pane", ["-t", &self.session, "-p", "-e"]);
    }

    // capture the viewport shifted `scroll` lines into the history. line 0 is
    // the top of the visible pane, and negative lines are in the history.
    let height = self.size.lock().height as isize;
    let start = -(scroll as isize);
    let end = height - 1 - scroll as isize;

    tmux_command(
      "capture-pane",
      [
        "-t",
        &self.session,
        "-p",
        "-e",
        "-S",
        &start.to_string(),
        "-E",
        &end.to_string(),
      ],
    )
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

    let mut display_info: DisplayInfo = serde_json::from_slice(&content)
      .with_context(|| format!("Failed to parse: {}", String::from_utf8_lossy(&content)))?;

    // hide the cursor while scrolled into the history
    if *self.scroll.lock() > 0 {
      display_info.cursor = Point {
        x: usize::MAX,
        y: usize::MAX,
      };
    }

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
