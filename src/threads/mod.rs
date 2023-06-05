use std::thread::{self, JoinHandle};

use anyhow::Result;

mod quit;
mod keys;
mod refresh;
mod resize;

pub use self::{
  quit::Quit,
  keys::Keys,
  refresh::Refresh,
  resize::Resize,
};

pub trait Spawn: Send {
  fn run(&self) -> Result<()>;

  fn spawn(self) -> JoinHandle<Result<()>>
  where
    Self: Sized + 'static,
  {
    thread::spawn(move || self.run())
  }
}
