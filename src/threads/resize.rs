use std::sync::Arc;

use anyhow::Result;
use parking_lot::Mutex;

use crate::{
  fifo::Fifo,
  geometry::Size,
  threads::{Quit, Spawn},
  tmux::Tmux,
};

pub struct Resize {
  padding: usize,
  tmux: Tmux,
  size: Arc<Mutex<Size>>,
  resize_fifo: Fifo,
  quit: Quit,
}

impl Resize {
  pub fn new(padding: usize, tmux: Tmux, size: Arc<Mutex<Size>>, resize_fifo: Fifo, quit: Quit) -> Self {
    println!("resize fifo is {:?}", resize_fifo.path);

    Self {
      padding,
      tmux,
      size,
      resize_fifo,
      quit,
    }
  }
}

impl Spawn for Resize {
  fn run(&self) -> Result<()> {
    while !self.quit.is_quit() {
      println!("resize");

      let new_size: Size = serde_json::from_str(&self.resize_fifo.read()?)?;
      let new_size = new_size.padded(self.padding)?;

      *self.size.lock() = new_size;

      // TODO: trigger a refresh
      self.tmux.resize_window(new_size)?;
    }

    Ok(())
  }
}

impl Drop for Resize {
  fn drop(&mut self) {
    self.quit.quit();
  }
}
