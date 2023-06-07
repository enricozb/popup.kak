use std::{
  sync::mpsc::{self, Receiver, Sender},
  thread::{self, JoinHandle},
  time::Duration,
};

use anyhow::Result;

use super::{Spawn, Step};
use crate::{buffer::Buffer, escape, kakoune::Kakoune, tmux::Tmux};

pub struct Refresh {
  pub sender: Sender<()>,
  receiver: Receiver<()>,

  kakoune: Kakoune,
  tmux: Tmux,
  title: String,

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
      title: title.unwrap_or_default(),

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

    let buffer = Buffer::new(self.tmux.display_info()?, self.tmux.capture_pane()?);
    let markup = escape::kak(buffer.markup()?);
    let title = escape::kak(&self.title);

    self
      .kakoune
      .eval(format!("info -style modal -title {title} -markup {markup}"))?;

    Ok(Step::Next)
  }
}
