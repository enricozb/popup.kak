use std::process::Stdio;

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

  async fn command(&self, command: &[u8]) -> Result<()> {
    let mut child = TokioCommand::new("kak")
      .args(["-p", &self.session])
      .stdin(Stdio::piped())
      .spawn()?;

    child
      .stdin
      .take()
      .ok_or(anyhow::anyhow!("no stdin"))?
      .write_all(command)
      .await?;

    Ok(())
  }

  pub async fn debug(&self, message: String) -> Result<()> {
    self.command(format!("echo -debug 'kak-popup:' %§{message}§").as_bytes()).await?;

    Ok(())
  }

  pub async fn eval(&self, command: String) -> Result<()> {
    self
      .command(format!("evaluate-commands -no-hooks -client '{}' %§{command}§", self.client).as_bytes())
      .await?;

    Ok(())
  }
}
