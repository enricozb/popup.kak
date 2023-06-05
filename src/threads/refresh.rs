use std::{sync::Arc, thread, time::Duration};

use anyhow::Result;
use parking_lot::Mutex;

use super::Spawn;
use crate::{escape, geometry::Size, kakoune::Kakoune, threads::Quit, tmux::Tmux};

pub struct Refresh {
  kakoune: Kakoune,
  tmux: Tmux,
  title: Option<String>,
  size: Arc<Mutex<Size>>,
  quit: Quit,
}

impl Refresh {
  const RATE: Duration = Duration::from_millis(100);

  pub fn new(kakoune: Kakoune, tmux: Tmux, title: Option<String>, size: Arc<Mutex<Size>>, quit: Quit) -> Self {
    Self {
      kakoune,
      tmux,
      title,
      size,
      quit,
    }
  }

  fn refresh(&self) -> Result<()> {
    let content = self.tmux.capture_pane()?;
    let width = self.size.lock().width;

    // strip the trailing newline
    let output = String::from_utf8_lossy(&content[..content.len() - 1]);

    let mut output: Vec<&str> = output.rsplitn(2, '\n').collect();
    output.reverse();

    let last_line;
    if let Some(last) = output.last_mut() {
      last_line = format!("{last:<width$}");
      *last = &last_line;
    }

    let title = if let Some(title) = &self.title {
      let title = escape::kak(format!("{title}: (<c-space> to exit)"));
      format!("-title {title}")
    } else {
      String::new()
    };

    let output = escape::kak(output.join("\n"));

    self.kakoune.eval(format!("info -style modal {title} {output}"))?;

    Ok(())
  }
}

impl Drop for Refresh {
  fn drop(&mut self) {
    self.quit.quit();
  }
}

impl Spawn for Refresh {
  fn run(&self) -> Result<()> {
    while !self.quit.is_quit() {
      println!("refresh");

      self.refresh()?;

      thread::sleep(Self::RATE);
    }

    Ok(())
  }
}
