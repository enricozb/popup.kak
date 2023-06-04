use std::path::PathBuf;

use anyhow::Result;
use nix::{sys::stat::Mode, unistd};
use tempfile::TempDir;
use tokio::fs as tokio_fs;

pub struct Fifo {
  pub path: PathBuf,

  _tempdir: TempDir,
}

impl Fifo {
  pub fn new(name: &str) -> Result<Self> {
    let tempdir = TempDir::new()?;

    let path = tempdir.path().join(name);
    unistd::mkfifo(&path, Mode::S_IRUSR | Mode::S_IWUSR)?;

    Ok(Self {
      path,
      _tempdir: tempdir,
    }) }

  pub fn path_str(&self) -> Result<&str> {
    self
      .path
      .to_str()
      .ok_or_else(|| anyhow::anyhow!("path to_str: {:?}", self.path))
  }

  pub async fn read(&self) -> Result<String> {
    Ok(tokio_fs::read_to_string(&self.path).await?)
  }

  pub async fn write(&self, contents: impl AsRef<[u8]>) -> Result<()> {
    Ok(tokio_fs::write(&self.path, contents).await?)
  }
}
