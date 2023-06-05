use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use tempfile::TempDir;

use crate::{args::OnErr, escape, kakoune::Kakoune};

pub struct Capture {
  kak_script: Option<String>,
  on_err: OnErr,
  status: Option<PathBuf>,
  stdout: Option<PathBuf>,
  stderr: Option<PathBuf>,
  _tempdir: TempDir,
}

impl Capture {
  pub fn new(kak_script: Option<String>, on_err: OnErr) -> Result<Self> {
    let tempdir = TempDir::new()?;

    let status = if Self::should_capture_status(on_err) {
      Some(tempdir.path().join("status"))
    } else {
      None
    };

    let stderr = if Self::should_capture_stderr(on_err) {
      Some(tempdir.path().join("stderr"))
    } else {
      None
    };

    let stdout = if kak_script.is_some() {
      Some(tempdir.path().join("stdout"))
    } else {
      None
    };

    Ok(Self {
      kak_script,
      on_err,
      status,
      stdout,
      stderr,
      _tempdir: tempdir,
    })
  }

  fn should_capture_status(on_err: OnErr) -> bool {
    match on_err {
      OnErr::Warn | OnErr::Dismiss => true,
      OnErr::Ignore => false,
    }
  }

  fn should_capture_stderr(on_err: OnErr) -> bool {
    match on_err {
      OnErr::Warn => true,
      OnErr::Ignore | OnErr::Dismiss => false,
    }
  }

  pub fn command(&self, command: &str, args: &[String], keys_fifo_path: &str) -> String {
    let save_status = self
      .status
      .as_ref()
      .map(|status| {
        let status = escape::bash(status.to_string_lossy());
        format!("; echo $? >{status}")
      })
      .unwrap_or_default();

    let save_stdout = self
      .stdout
      .as_ref()
      .map(|stdout| {
        let stdout = escape::bash(stdout.to_string_lossy());
        format!(">{stdout}")
      })
      .unwrap_or_default();

    let save_stderr = self
      .stderr
      .as_ref()
      .map(|stderr| {
        let stderr = escape::bash(stderr.to_string_lossy());
        format!("2> >(tee {stderr} >&2)")
      })
      .unwrap_or_default();

    let args = args.iter().map(escape::bash).collect::<Vec<String>>().join(" ");
    let keys_fifo_path = escape::bash(keys_fifo_path);

    let command = escape::bash(format!(
      "{command} {args} {save_stdout} {save_stderr} {save_status}; echo quit > {keys_fifo_path}"
    ));

    format!("bash -c {command}")
  }

  pub fn handle_output(&self, kakoune: &Kakoune) -> Result<()> {
    let on_err = escape::kak(format!("{}", self.on_err));

    let status = self
      .status
      .as_ref()
      .map(fs::read_to_string)
      .transpose()
      .context("status")?
      .unwrap_or_default();

    let stdout = self
      .stdout
      .as_ref()
      .map(fs::read_to_string)
      .transpose()
      .context("stdout")?
      .unwrap_or_default();

    let stderr = self
      .stderr
      .as_ref()
      .map(fs::read_to_string)
      .transpose()
      .context("stderr")?
      .unwrap_or_default();

    let status = escape::kak(status.trim());
    let stdout = escape::kak(stdout.trim());
    let stderr = escape::kak(stderr.trim());

    let kak_script = escape::kak(self.kak_script.clone().unwrap_or_default());

    kakoune.eval(format!(
      "popup-handle-output {on_err} {status} {stdout} {stderr} {kak_script}"
    ))?;

    Ok(())
  }
}
