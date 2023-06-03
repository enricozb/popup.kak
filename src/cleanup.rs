use std::path::{PathBuf, Path};

pub struct Cleanup {
  pub kak_script: String,
  pub stdout: PathBuf,
  pub stderr: PathBuf,
}

impl Cleanup {
  pub fn new(kak_script: String, dir: &Path) -> Self {
    Self {
      kak_script,
      stdout: dir.join("stdout"),
      stderr: dir.join("stderr"),
    }
  }
}
