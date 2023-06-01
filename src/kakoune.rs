use std::{fmt::Display, process::Stdio};

use anyhow::Result;
use tokio::{io::AsyncWriteExt, process::Command as TokioCommand};

pub struct Kakoune {
  session: String,
  client: String,
}

impl Kakoune {
  pub fn new(session: String, client: String) -> Self {
    Self { session, client }
  }

  async fn command<S: AsRef<[u8]>>(&self, command: S) -> Result<()> {
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

  pub async fn debug<D: Display>(&self, message: D) -> Result<()> {
    self
      .command(format!("echo -debug 'kak-popup:' %§{message}§").as_bytes())
      .await?;

    Ok(())
  }

  pub async fn eval<D: Display>(&self, command: D) -> Result<()> {
    self
      .command(format!("evaluate-commands -no-hooks -client '{}' %§{command}§", self.client).as_bytes())
      .await?;

    Ok(())
  }

  pub async fn exec<D: Display>(&self, keys: D) -> Result<()> {
    self
      .command(format!("execute-keys -client '{}' %§{keys}§", self.client).as_bytes())
      .await?;

    Ok(())
  }
}
