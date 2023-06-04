use clap::{Args as SubcommandArgs, Parser, Subcommand, ValueEnum};
use strum::Display;

#[derive(Clone, Copy, Default, Debug, Display, ValueEnum)]
#[strum(serialize_all = "snake_case")]
pub enum OnErr {
  /// Show a modal with stderr.
  Warn,
  /// Dismiss modal and don't run any provided KAK_SCRIPT.
  #[default]
  Dismiss,
  /// Ignore status and always run any provided KAK_SCRIPT.
  Ignore,
}

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

  /// What to do on non-zero exit status.
  #[arg(long, default_value_t)]
  pub on_err: OnErr,

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
