mod popup;

use anyhow::Result;
use clap::Parser;

use self::popup::Popup;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  /// The command to execute.
  #[arg(long)]
  command: String,

  /// The fifo to send commands to kakoune.
  #[arg(long, value_name = "URL")]
  command_fifo: String,

  /// The fifo to read keys from kakoune.
  #[arg(long, value_name = "URL")]
  keys_fifo: String,

  /// The height of the kakoune window.
  #[arg(long)]
  height: usize,

  /// The width of the kakoune window.
  #[arg(long)]
  width: usize,
}

fn main() -> Result<()> {
  let args = Args::parse();
  let popup = Popup::new(args)?;

  println!("starting tmux session {}...", popup.tmux_session);

  popup.start()?;

  Ok(())
}
