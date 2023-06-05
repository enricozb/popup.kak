use std::sync::Arc;

use anyhow::Result;
use parking_lot::Mutex;

use super::{Spawn, Step};
use crate::{fifo::Fifo, geometry::Size, tmux::Tmux};

pub struct Resize {
  padding: usize,
  tmux: Tmux,
  size: Arc<Mutex<Size>>,
  resize_fifo: Fifo,
}

impl Resize {
  pub fn new(padding: usize, tmux: Tmux, size: Arc<Mutex<Size>>, resize_fifo: Fifo) -> Self {
    Self {
      padding,
      tmux,
      size,
      resize_fifo,
    }
  }
}

impl Spawn for Resize {
  const NAME: &'static str = "resize";

  fn step(&self) -> Result<Step> {
    let new_size: Size = serde_json::from_str(&self.resize_fifo.read()?)?;
    let new_size = new_size.padded(self.padding)?;

    *self.size.lock() = new_size;

    // TODO: trigger a refresh
    self.tmux.resize_window(new_size)?;

    Ok(Step::Next)
  }
}
