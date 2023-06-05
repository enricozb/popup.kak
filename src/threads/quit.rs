use std::sync::Arc;

use parking_lot::{Condvar, Mutex};

#[derive(Clone)]
pub struct Quit {
  inner: Arc<(Mutex<bool>, Condvar)>,
}

impl Quit {
  pub fn new() -> Self {
    Self {
      inner: Arc::new((Mutex::new(false), Condvar::new())),
    }
  }

  pub fn quit(&self) {
    let (mutex, condvar) = &*self.inner;
    *mutex.lock() = true;
    condvar.notify_one();
  }

  pub fn wait(&self) {
    let (mutex, condvar) = &*self.inner;
    condvar.wait(&mut mutex.lock());
  }

  pub fn is_quit(&self) -> bool {
    *self.inner.0.lock()
  }
}
