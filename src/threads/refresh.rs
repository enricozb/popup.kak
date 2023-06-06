use std::{
  sync::mpsc::{self, Receiver, Sender},
  thread::{self, JoinHandle},
  time::Duration,
};

use anyhow::Result;

use super::{Spawn, Step};
use crate::{escape, kakoune::Kakoune, tmux::Tmux};

pub struct Refresh {
  pub sender: Sender<()>,
  receiver: Receiver<()>,

  kakoune: Kakoune,
  tmux: Tmux,
  title: Option<String>,

  _events: JoinHandle<Result<()>>,
}

impl Refresh {
  const RATE: Duration = Duration::from_millis(100);

  pub fn new(kakoune: Kakoune, tmux: Tmux, title: Option<String>) -> Self {
    let (sender, receiver) = mpsc::channel();

    Self {
      sender: sender.clone(),
      receiver,

      kakoune,
      tmux,
      title,

      _events: thread::spawn(move || loop {
        sender.send(())?;
        thread::sleep(Self::RATE);
      }),
    }
  }
}

impl Spawn for Refresh {
  const NAME: &'static str = "refresh";

  fn step(&self) -> Result<Step> {
    self.receiver.recv()?;

    let content = self.tmux.capture_pane()?;
    let width = self.tmux.display_dimensions()?.width;

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

    Ok(Step::Next)
  }
}
