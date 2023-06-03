mod cleanup;
mod kakoune;
mod popup;
mod tmux;

use anyhow::Result;
use clap::Parser;

use self::popup::Popup;

#[derive(Parser)]
#[command(author, version)]
pub struct Args {
  /// The kakoune session to send commands to.
  #[arg(long)]
  kak_session: String,

  /// The kakoune client to send commands to.
  #[arg(long)]
  kak_client: String,

  /// The kakoune script to execute on completion.
  #[arg(long)]
  kak_script: Option<String>,

  /// The title of the popup.
  #[arg(long)]
  title: String,

  /// The command to execute within the popup.
  #[arg(long)]
  command: String,

  /// The height of the kakoune window.
  #[arg(long)]
  height: usize,

  /// The width of the kakoune window.
  #[arg(long)]
  width: usize,
}

fn main() -> Result<()> {
  let args = Args::try_parse()?;
  let popup = Popup::new(
    args.kak_session,
    args.kak_client,
    args.kak_script.filter(|script| script.trim().len() > 0),
    args.title,
    args.command,
    args.height,
    args.width,
  )?;

  popup.start()?;

  Ok(())
}
