use std::{
  io::Write,
  process::{Command, Stdio},
};

use anyhow::Result;

use crate::escape;

#[derive(Clone)]
pub struct Kakoune {
  session: String,
  client: String,
}

impl Kakoune {
  pub fn new(session: String, client: String) -> Self {
    Self { session, client }
  }

  fn command(&self, command: impl AsRef<[u8]>) -> Result<()> {
    let mut child = Command::new("kak")
      .args(["-p", &self.session])
      .stdin(Stdio::piped())
      .spawn()?;

    child
      .stdin
      .take()
      .ok_or(anyhow::anyhow!("no stdin"))?
      .write_all(command.as_ref())?;

    Ok(())
  }

  pub fn debug(&self, message: impl AsRef<str>) -> Result<()> {
    let message = escape::kak(message);

    self.command(format!("echo -debug 'kak-popup:' {message}").as_bytes())?;

    Ok(())
  }

  pub fn eval(&self, command: impl AsRef<str>) -> Result<()> {
    let command = escape::kak(command);

    self.command(format!("evaluate-commands -client '{}' {command}", self.client).as_bytes())?;

    Ok(())
  }

  pub fn debug_on_error(&self, f: impl FnOnce() -> Result<()>) -> Result<()> {
    if let Err(err) = f() {
      self.debug(format!("error: {err:?}"))?;
    }

    Ok(())
  }
}
