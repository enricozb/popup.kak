mod kakoune;
mod popup;

use anyhow::{Context, Result};
use clap::Parser;

use self::popup::Popup;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  /// The command to execute.
  #[arg(long)]
  command: String,

  /// The kakoune session to send commands to.
  #[arg(long)]
  kak_session: String,

  /// The kakoune client to send commands to.
  #[arg(long)]
  kak_client: String,

  /// The height of the kakoune window.
  #[arg(long)]
  height: usize,

  /// The width of the kakoune window.
  #[arg(long)]
  width: usize,
}

fn wrapped_main() -> Result<()> {
  let args = Args::try_parse()?;
  let popup = Popup::new(args)?;

  popup.start()?;

  Ok(())
}

fn main() -> Result<()> {
  wrapped_main().context("kak-popup")
}
