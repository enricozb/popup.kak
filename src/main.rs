mod cleanup;
mod escape;
mod kakoune;
mod popup;
mod tmux;

use anyhow::Result;
use clap::Parser;

use self::{kakoune::Kakoune, popup::Popup};

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
  title: Option<String>,

  /// The height of the kakoune window.
  #[arg(long)]
  height: usize,

  /// The width of the kakoune window.
  #[arg(long)]
  width: usize,

  /// Show warning modal if COMMAND has non-zero exit status.
  #[arg(long)]
  warn: bool,

  /// The command to execute within the popup.
  command: String,

  /// Any arguments to the command.
  args: Vec<String>,
}

fn main() -> Result<()> {
  let args = Args::parse();
  let kakoune = Kakoune::new(args.kak_session, args.kak_client);

  let popup = Popup::new(
    kakoune,
    args.kak_script,
    args.title,
    &args.command,
    &args.args,
    args.height,
    args.width,
  )?;

  popup.start()?;

  Ok(())
}
