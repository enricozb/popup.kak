use std::{fs, path::PathBuf};

use anyhow::Result;
use nix::{sys::stat::Mode, unistd};
use tempfile::TempDir;

pub struct Fifo {
  pub path: PathBuf,

  _tempdir: Option<TempDir>,
}

impl Clone for Fifo {
  fn clone(&self) -> Self {
    Self {
      path: self.path.clone(),
      _tempdir: None,
    }
  }
}

impl Fifo {
  pub fn new(name: &str) -> Result<Self> {
    let tempdir = TempDir::new()?;

    let path = tempdir.path().join(name);
    unistd::mkfifo(&path, Mode::S_IRUSR | Mode::S_IWUSR)?;

    Ok(Self {
      path,
      _tempdir: Some(tempdir),
    })
  }

  pub fn path_str(&self) -> Result<&str> {
    self
      .path
      .to_str()
      .ok_or_else(|| anyhow::anyhow!("path to_str: {:?}", self.path))
  }

  pub fn read(&self) -> Result<String> {
    Ok(fs::read_to_string(&self.path)?)
  }

  pub fn write(&self, contents: impl AsRef<[u8]>) -> Result<()> {
    Ok(fs::write(&self.path, contents)?)
  }
}
