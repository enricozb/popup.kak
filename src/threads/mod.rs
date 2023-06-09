use std::thread::{self, JoinHandle};

use anyhow::Result;

mod keys;
mod quit;
mod refresh;
mod resize;

pub use self::{keys::Keys, quit::Quit, refresh::Refresh, resize::Resize};
use crate::kakoune::Kakoune;

pub enum Step {
  Next,
  Quit,
}

pub trait Spawn {
  const NAME: &'static str;

  fn step(&self) -> Result<Step>;

  fn spawn(self, kakoune: Kakoune, quit: Quit) -> JoinHandle<()>
  where
    Self: Send + Sized + 'static,
  {
    thread::spawn(move || {
      while !quit.is_quit() {
        match self.step() {
          Ok(Step::Next) => (),

          Ok(Step::Quit) => {
            let _ignore = kakoune.debug(format!("{}::step: quitting", Self::NAME));
            quit.quit();
          }

          Err(err) => {
            let _ignore = kakoune.debug(format!("{}::step: {err:?}", Self::NAME));
            quit.quit();
          }
        }
      }
    })
  }
}
