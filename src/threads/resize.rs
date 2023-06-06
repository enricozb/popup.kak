use std::sync::mpsc::Sender;

use anyhow::Result;

use super::{Spawn, Step};
use crate::{fifo::Fifo, geometry::Size, tmux::Tmux};

pub struct Resize {
  padding: usize,
  tmux: Tmux,
  resize_fifo: Fifo,
  refresh: Sender<()>,
}

impl Resize {
  pub fn new(padding: usize, tmux: Tmux, resize_fifo: Fifo, refresh: Sender<()>) -> Self {
    Self {
      padding,
      tmux,
      resize_fifo,
      refresh,
    }
  }
}

impl Spawn for Resize {
  const NAME: &'static str = "resize";

  fn step(&self) -> Result<Step> {
    let new_size: Size = serde_json::from_str(&self.resize_fifo.read()?)?;
    let new_size = new_size.padded(self.padding)?;

    self.tmux.resize_window(new_size)?;
    self.refresh.send(())?;

    Ok(Step::Next)
  }
}
