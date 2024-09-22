mod args;
mod buffer;
mod capture;
mod escape;
mod fifo;
mod geometry;
mod kakoune;
mod popup;
mod threads;
mod tmux;

use std::{env, fs::File, os::unix::ffi::OsStringExt, thread, time::Duration};

use anyhow::{Context, Result};
use clap::Parser;
use daemonize::Daemonize;
use tempfile::TempDir;

use self::{
  args::{Args, Command, Popup as PopupArgs},
  capture::Capture,
  fifo::Fifo,
  kakoune::Kakoune,
  popup::Popup,
};

fn daemonize() -> Result<TempDir> {
  let tempdir = TempDir::new()?;

  Daemonize::new()
    .working_directory(env::current_dir()?)
    .stdout(File::create(tempdir.path().join("stdout"))?)
    .stderr(File::create(tempdir.path().join("stderr"))?)
    .pid_file(tempdir.path().join("pid"))
    .start()?;

  Ok(tempdir)
}

fn init() {
  println!("{kak_script}", kak_script = include_str!("../rc/popup.kak"));
}

fn popup(args: PopupArgs) -> Result<()> {
  let _: Option<TempDir> = if args.daemonize { Some(daemonize()?) } else { None };

  let kakoune = Kakoune::new(args.kak_session, args.kak_client, args.debug);

  kakoune.debug_on_error(|| {
    let capture = Capture::new(args.kak_script, args.on_err)?;
    let keys_fifo = Fifo::new("keys")?;
    let command = capture.command(&args.command, &args.args, args.input.map(OsStringExt::into_vec))?;

    Popup::new(
      kakoune.clone(),
      keys_fifo,
      args.title,
      args.height,
      args.width,
      args.padding,
      &command,
    )
    .context("Popup::new")?
    .show()
    .context("Popup::show")?;

    capture.handle_output(&kakoune).context("Capture::handle_output")?;

    // allow any remaining fifos to be flushed
    thread::sleep(Duration::from_secs(1));

    Ok(())
  })?;

  Ok(())
}

fn main() -> Result<()> {
  let args = Args::parse();

  match args.command {
    Command::Init => init(),
    Command::Popup(mut args) => {
      // A heuristic to ignore padding if it's too large relative to height or width
      if 3 * args.padding >= args.height || 3 * args.padding >= args.width {
        args.padding = 4
      }

      popup(args)?;
    }
  }

  Ok(())
}
