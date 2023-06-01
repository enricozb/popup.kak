use std::{
  fs::File,
  path::PathBuf,
  process::Command,
  sync::atomic::{AtomicBool, AtomicUsize, Ordering},
  time::{Duration, SystemTime},
};

use anyhow::{Context, Result};
use daemonize::Daemonize;
use nix::{sys::stat::Mode, unistd};
use tempfile::TempDir;
use tokio::{
  fs as tokio_fs,
  process::Command as TokioCommand,
  time::{self as tokio_time},
};

use crate::{kakoune::Kakoune, Args};

pub struct Popup {
  tmux_session: String,
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
      tmux_session: Self::new_session(width, height, args.command)?,
      kakoune: Kakoune::new(args.kak_session, args.kak_client),
      width: AtomicUsize::new(width),
      height: AtomicUsize::new(height),
      tempdir,
      fifo,
      quit: AtomicBool::new(false),
    })
  }

  fn new_session(width: usize, height: usize, command: String) -> Result<String> {
    let duration_since_epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let session_name = duration_since_epoch.as_nanos().to_string();

    let status = Command::new("tmux")
      .args([
        "new-session",
        "-d",
        "-s",
        &session_name,
        "-x",
        &width.to_string(),
        "-y",
        &height.to_string(),
      ])
      .status()
      .context("tmux")?;

    if !status.success() {
      return Err(anyhow::anyhow!(
        "tmux new-session exited with non-zero status: {status}"
      ));
    }

    let status = Command::new("tmux")
      .args(["send-keys", "-t", &session_name, &command, "Enter"])
      .status()
      .context("tmux")?;

    if !status.success() {
      return Err(anyhow::anyhow!("tmux send-keys exited with non-zero status: {status}"));
    }

    Ok(session_name)
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

      let output = TokioCommand::new("tmux")
        .args(["capture-pane", "-p", "-t", &self.tmux_session])
        .output()
        .await
        .context("tmux capture-pane")?;

      if !output.status.success() {
        return Err(anyhow::anyhow!(
          "tmux capture-pane exited with non-zero status: {}",
          output.status
        ));
      }

      // TODO: if output.stderr not empty, send to kakoune debug

      let width = self.width.load(Ordering::Relaxed);
      let height = self.height.load(Ordering::Relaxed);

      let output = String::from_utf8_lossy(&output.stdout);
      if output != last_output {
        let mut output: Vec<String> = output
          .split("\n")
          .into_iter()
          .map(|line| format!("{:<width$}", line, width = width))
          .collect();

        if output.len() < height {
          output.extend(vec![String::new(); height - output.len()])
        }

        let output = output.join("\n").replace("'", "''");

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

    self.kakoune.debug(format!("sending key: {key}")).await?;

    let output = TokioCommand::new("tmux")
      .args(["send-keys", "-t", &self.tmux_session, key])
      .output()
      .await
      .context("tmux capture-pane")?;

    if !output.status.success() {
      self
        .kakoune
        .debug(format!("tmux send-keys exited with non-zero status: {}", output.status))
        .await?;
    }

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
    self
      .kakoune
      .debug(format!(
        "started tmux at {} with width={:?} height={:?}",
        self.tmux_session, self.width, self.height
      ))
      .await?;

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

impl Drop for Popup {
  fn drop(&mut self) {
    let status = Command::new("tmux")
      .args(["kill-session", "-t", &self.tmux_session])
      .status()
      .expect("tmux kill-session");

    assert!(
      status.success(),
      "tmux kill-session exited with non-zero status: {status}"
    );
  }
}
