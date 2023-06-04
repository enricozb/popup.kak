mod args;
mod capture;
mod escape;
mod kakoune;
mod popup;
mod tmux;

use anyhow::Result;
use clap::Parser;

use self::{
  args::{Args, Command, Popup as PopupArgs},
  capture::Capture,
  kakoune::Kakoune,
  popup::Popup,
};

fn init() {
  let kak_script = include_str!("../rc/popup.kak");

  println!("{kak_script}");
}

fn popup(args: PopupArgs) -> Result<()> {
  let kakoune = Kakoune::new(args.kak_session, args.kak_client);
  let capture = Capture::new(args.kak_script, args.on_err)?;
  let command = capture.command(&args.command, &args.args);

  Popup::new(&kakoune, args.title, args.height, args.width, &command)?.show()?;

  capture.handle_output(&kakoune)?;

  Ok(())
}

fn main() -> Result<()> {
  let args = Args::parse();

  match args.command {
    Command::Init => init(),
    Command::Popup(args) => popup(args)?,
  }

  Ok(())
}
