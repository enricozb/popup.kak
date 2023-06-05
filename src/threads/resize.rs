use std::sync::{mpsc::Sender, Arc};

use anyhow::Result;
use parking_lot::Mutex;

use super::{Spawn, Step};
use crate::{fifo::Fifo, geometry::Size, tmux::Tmux};

pub struct Resize {
  padding: usize,
  tmux: Tmux,
  size: Arc<Mutex<Size>>,
  resize_fifo: Fifo,
  refresh: Sender<()>,
}

impl Resize {
  pub fn new(padding: usize, tmux: Tmux, size: Arc<Mutex<Size>>, resize_fifo: Fifo, refresh: Sender<()>) -> Self {
    Self {
      padding,
      tmux,
      size,
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

    *self.size.lock() = new_size;

    self.tmux.resize_window(new_size)?;
    self.refresh.send(())?;

    Ok(Step::Next)
  }
}
