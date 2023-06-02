mod kakoune;
mod popup;
mod tmux;

use anyhow::Result;
use clap::Parser;

use self::popup::Popup;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  /// The title of the popup.
  #[arg(long)]
  title: String,

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

fn main() -> Result<()> {
  let args = Args::try_parse()?;
  let popup = Popup::new(args)?;

  popup.start()?;

  Ok(())
}
