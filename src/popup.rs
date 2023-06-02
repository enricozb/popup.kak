use std::{
  fs::File,
  path::PathBuf,
  sync::atomic::{AtomicBool, AtomicUsize, Ordering},
  time::Duration,
};

use anyhow::Result;
use daemonize::Daemonize;
use nix::{sys::stat::Mode, unistd};
use tempfile::TempDir;
use tokio::{fs as tokio_fs, time as tokio_time};

use crate::{kakoune::Kakoune, tmux::Tmux, Args};

pub struct Popup {
  tmux: Tmux,
  kakoune: Kakoune,

  title: String,
  width: AtomicUsize,
  height: AtomicUsize,

  tempdir: TempDir,
  fifo: PathBuf,

  quit: AtomicBool,
}

impl Popup {
  pub fn new(args: Args) -> Result<Self> {
    let width = args.width.checked_sub(15).ok_or(anyhow::anyhow!("width too small"))?;
    let height = args.height.checked_sub(15).ok_or(anyhow::anyhow!("height too small"))?;

    let tempdir = TempDir::new()?;
    let fifo = tempdir.path().join("kak-popup-commands");

    println!("{}", fifo.to_str().expect("fifo to_str"));

    unistd::mkfifo(&fifo, Mode::S_IRUSR | Mode::S_IWUSR)?;

    Ok(Self {
      tmux: Tmux::new(&args.command, width, height)?,
      kakoune: Kakoune::new(args.kak_session, args.kak_client),
      title: args.title,
      width: AtomicUsize::new(width),
      height: AtomicUsize::new(height),
      tempdir,
      fifo,
      quit: AtomicBool::new(false),
    })
  }

  fn daemonize(&self) -> Result<()> {
    let tempdir_path = self.tempdir.path();

    Daemonize::new()
      .stdout(File::create(tempdir_path.join("stdout"))?)
      .stderr(File::create(tempdir_path.join("stderr"))?)
      .pid_file(tempdir_path.join("pid"))
      .start()?;

    Ok(())
  }

  async fn refresh_loop(&self) -> Result<()> {
    let sleep_duration = Duration::from_millis(100);

    loop {
      if self.quit.load(Ordering::Relaxed) {
        return Ok(());
      }

      self.refresh().await?;

      tokio_time::sleep(sleep_duration).await;
    }
  }

  async fn refresh(&self) -> Result<()> {
    let content = self.tmux.capture_pane().await?;

    let width = self.width.load(Ordering::Relaxed);

    let output = String::from_utf8_lossy(&content[..content.len() - 1]);
    let mut output: Vec<&str> = output.rsplitn(2, '\n').collect();
    output.reverse();

    let last_line;
    if let Some(last) = output.last_mut() {
      last_line = format!("{last:<width$}");
      *last = &last_line;
    }

    let output = output.join("\n").replace('\'', "''");

    self
      .kakoune
      .eval(format!("info -style modal -title '{}' '{output}'", self.title))
      .await?;

    Ok(())
  }

  async fn send_key(&self, key: &str) -> Result<()> {
    let mut key = match key {
      "<esc>" => "Escape",
      "<ret>" => "Enter",
      "<tab>" => "Tab",
      "<s-tab>" => "BTab",
      "<space>" => "Space",
      "<backspace>" => "BSpace",
      key => key,
    };

    let new_key;
    if key.starts_with("<c-") {
      new_key = format!("C-{}", &key[3..key.len() - 1]);
      key = &new_key;
    }

    self.tmux.send_keys(key).await?;
    self.refresh().await?;

    Ok(())
  }

  async fn event_loop(&self) -> Result<()> {
    loop {
      let event = tokio_fs::read_to_string(&self.fifo).await?;
      let event = event.trim();

      self.kakoune.debug(format!("received '{event}'")).await?;

      if event == "quit" {
        self.quit.store(true, Ordering::Relaxed);
        return Ok(());
      }

      if event.starts_with("resize") {
        match event.split(' ').collect::<Vec<&str>>().as_slice() {
          // TODO: handle resize
          [_, width, height] => (),
          _ => {
            self.kakoune.debug(format!("invalid resize: {event}")).await?;
          }
        }
        continue;
      }

      self.send_key(event).await?;
    }
  }

  #[tokio::main]
  async fn run(&self) -> Result<()> {
    // TODO: find a way such that, we tell kakoune to cancel the modal and on-key
    //       as soon as either of the futures return, however we still want to
    //       wait for everything here and report errors (if any).
    if let Err(err) = tokio::try_join!(self.refresh_loop(), self.event_loop()) {
      self.kakoune.debug(format!("error: {err:?}")).await?;
    }

    self.kakoune.eval("popup-close").await?;

    Ok(())
  }

  pub fn start(&self) -> Result<()> {
    self.daemonize()?;
    self.run()?;

    Ok(())
  }
}
