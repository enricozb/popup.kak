use std::{
  io::Write,
  process::{Command, Stdio},
};

use anyhow::Result;
use tokio::{io::AsyncWriteExt, process::Command as TokioCommand};

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

  fn sync_command(&self, command: impl AsRef<[u8]>) -> Result<()> {
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

  async fn command(&self, command: impl AsRef<[u8]>) -> Result<()> {
    let mut child = TokioCommand::new("kak")
      .args(["-p", &self.session])
      .stdin(Stdio::piped())
      .spawn()?;

    child
      .stdin
      .take()
      .ok_or(anyhow::anyhow!("no stdin"))?
      .write_all(command.as_ref())
      .await?;

    Ok(())
  }

  pub fn sync_debug(&self, message: impl AsRef<str>) -> Result<()> {
    let message = escape::kak(message);

    self.sync_command(format!("echo -debug 'kak-popup:' {message}").as_bytes())?;

    Ok(())
  }

  pub async fn debug(&self, message: impl AsRef<str>) -> Result<()> {
    let message = escape::kak(message);

    self
      .command(format!("echo -debug 'kak-popup:' {message}").as_bytes())
      .await?;

    Ok(())
  }

  pub async fn eval(&self, command: impl AsRef<str>) -> Result<()> {
    let command = escape::kak(command);

    self
      .command(format!("evaluate-commands -client '{}' {command}", self.client).as_bytes())
      .await?;

    Ok(())
  }

  pub async fn exec(&self, keys: impl AsRef<str>) -> Result<()> {
    let keys = escape::kak(keys);

    self
      .command(format!("execute-keys -client '{}' {keys}", self.client).as_bytes())
      .await?;

    Ok(())
  }

  pub fn debug_on_error(&self, f: impl FnOnce() -> Result<()>) -> Result<()> {
    if let Err(err) = f() {
      self.sync_debug(format!("error: {err:?}"))?;
    }

    Ok(())
  }
}
