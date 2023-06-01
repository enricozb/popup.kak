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

  width: AtomicUsize,
  height: AtomicUsize,

  tempdir: TempDir,
  fifo: PathBuf,

  quit: AtomicBool,
}

impl Popup {
  pub fn new(args: Args) -> Result<Self> {
    let width = args.width.checked_sub(8).ok_or(anyhow::anyhow!("width too small"))?;
    let height = args.height.checked_sub(8).ok_or(anyhow::anyhow!("height too small"))?;

    let tempdir = TempDir::new()?;
    let fifo = tempdir.path().join("kak-popup-commands");

    println!("{}", fifo.to_str().expect("fifo to_str"));

    unistd::mkfifo(&fifo, Mode::S_IRUSR | Mode::S_IWUSR)?;

    Ok(Self {
      tmux: Tmux::new(&args.command, width, height)?,
      kakoune: Kakoune::new(args.kak_session, args.kak_client),
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
    let sleep_duration = Duration::from_millis(500);

    let last_output = String::new();

    loop {
      if self.quit.load(Ordering::Relaxed) {
        return Ok(());
      }

      let content = self.tmux.capture_pane().await?;

      // TODO: if output.stderr not empty, send to kakoune debug

      let width = self.width.load(Ordering::Relaxed);
      let height = self.height.load(Ordering::Relaxed);

      let output = String::from_utf8_lossy(&content);
      if output != last_output {
        let mut output: Vec<String> = output.split('\n').map(|line| format!("{line:<width$}")).collect();

        if output.len() < height {
          output.extend(vec![String::new(); height - output.len()]);
        }

        let output = output.join("\n").replace('\'', "''");

        self.kakoune.eval(format!("info -style modal '{output}'")).await?;
      }

      tokio_time::sleep(sleep_duration).await;
    }
  }

  async fn send_key(&self, key: &str) -> Result<()> {
    let key = match key {
      "<esc>" => "Escape",
      "<ret>" => "Enter",
      "<tab>" => "Tab",
      "<space>" => "Space",
      "<backspace>" => "BSpace",
      key => key,
    };

    self.tmux.async_send_keys(key).await?;

    Ok(())
  }

  async fn event_loop(&self) -> Result<()> {
    loop {
      let event = tokio_fs::read_to_string(&self.fifo).await?;
      let event = event.trim();

      if event == "quit" {
        self.quit.store(true, Ordering::Relaxed);
        return Ok(());
      }

      if event.starts_with("resize") {
        // TODO: handle resize
        continue;
      }

      self.send_key(event).await?;
    }
  }

  #[tokio::main]
  async fn run(&self) -> Result<()> {
    let refresh_loop = self.refresh_loop();
    let event_loop = self.event_loop();

    let (refresh_res, event_res) = tokio::join!(refresh_loop, event_loop);

    refresh_res?;
    event_res?;

    self.kakoune.eval("info -style modal".to_string()).await?;

    Ok(())
  }

  pub fn start(&self) -> Result<()> {
    self.daemonize()?;
    self.run()?;

    Ok(())
  }
}