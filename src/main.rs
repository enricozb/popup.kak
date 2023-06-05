mod args;
mod capture;
mod escape;
mod fifo;
mod geometry;
mod kakoune;
mod popup;
mod threads;
mod tmux;

use std::fs::File;

use anyhow::{Context, Result};
use clap::Parser;
use daemonize::Daemonize;
use tempfile::TempDir;

use self::{
  args::{Args, Command, Popup as PopupArgs},
  capture::Capture,
  kakoune::Kakoune,
  popup::Popup,
};

fn daemonize() -> Result<TempDir> {
  let tempdir = TempDir::new()?;

  Daemonize::new()
    .stdout(File::create(tempdir.path().join("stdout"))?)
    .stderr(File::create(tempdir.path().join("stderr"))?)
    .pid_file(tempdir.path().join("pid"))
    .start()?;

  Ok(tempdir)
}

fn init() {
  let kak_script = include_str!("../rc/popup.kak");

  println!("{kak_script}");
}

fn popup(args: PopupArgs) -> Result<()> {
  let _: Option<TempDir> = if args.daemonize { Some(daemonize()?) } else { None };

  let kakoune = Kakoune::new(args.kak_session, args.kak_client);

  kakoune.debug_on_error(|| {
    let capture = Capture::new(args.kak_script, args.on_err)?;
    let command = capture.command(&args.command, &args.args);

    Popup::new(kakoune.clone(), args.title, args.height, args.width, &command)
      .context("Popup::new")?
      .show()
      .context("Popup::show")?;

    capture.handle_output(&kakoune).context("Capture::handle_output")?;

    Ok(())
  })?;

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
