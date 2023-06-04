use clap::{Args as SubcommandArgs, Parser, Subcommand};

#[derive(SubcommandArgs)]
pub struct Popup {
  /// The kakoune session to send commands to.
  #[arg(long)]
  pub kak_session: String,

  /// The kakoune client to send commands to.
  #[arg(long)]
  pub kak_client: String,

  /// The kakoune script to execute on completion.
  #[arg(long)]
  pub kak_script: Option<String>,

  /// The height of the kakoune window.
  #[arg(long)]
  pub height: usize,

  /// The width of the kakoune window.
  #[arg(long)]
  pub width: usize,

  /// Show warning modal if COMMAND has non-zero exit status.
  #[arg(long)]
  pub warn: bool,

  /// The title of the popup.
  #[arg(long)]
  pub title: Option<String>,

  /// The command to execute within the popup.
  pub command: String,

  /// Any arguments to the command.
  pub args: Vec<String>,
}

#[derive(Subcommand)]
pub enum Command {
  /// Outputs kak script to be used prior to any call to `popup`.
  Init,

  /// Starts a popup server instance.
  Popup(Popup),
}

#[derive(Parser)]
#[command(author, version)]
pub struct Args {
  #[command(subcommand)]
  pub command: Command,
}
