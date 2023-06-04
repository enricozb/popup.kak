use std::{
  fs::File,
  path::PathBuf,
  sync::atomic::{AtomicBool, AtomicUsize, Ordering},
  time::Duration,
};

use anyhow::{Context, Result};
use daemonize::Daemonize;
use nix::{sys::stat::Mode, unistd};
use tempfile::TempDir;
use tokio::{fs as tokio_fs, time as tokio_time};

use crate::{cleanup::Cleanup, escape, kakoune::Kakoune, tmux::Tmux};

pub struct Popup {
  tmux: Tmux,
  kakoune: Kakoune,
  cleanup: Option<Cleanup>,

  title: Option<String>,
  height: AtomicUsize,
  width: AtomicUsize,

  tempdir: TempDir,
  fifo: PathBuf,

  quit: AtomicBool,
}

impl Popup {
  const PADDING: usize = 16;

  pub fn new(
    kak_session: String,
    kak_client: String,
    kak_script: Option<String>,

    title: Option<String>,
    command: String,
    args: Vec<String>,
    height: usize,
    width: usize,
  ) -> Result<Self> {
    let kakoune = Kakoune::new(kak_session, kak_client);
    let tempdir = TempDir::new()?;

    let (command, cleanup) = if let Some(kak_script) = kak_script {
      let cleanup = Cleanup::new(kak_script, tempdir.path());
      let command = Self::wrap_command(&cleanup, &command, &args);

      (command, Some(cleanup))
    } else {
      let args: Vec<String> = args.iter().map(escape::bash).collect();
      let command = escape::bash(format!("{command} {}", args.join(" ")));
      (format!("bash -c {command}"), None)
    };

    kakoune.sync_debug(format!("command: {command}"))?;

    let height = height
      .checked_sub(Self::PADDING)
      .ok_or(anyhow::anyhow!("height too small"))?;

    let width = width
      .checked_sub(Self::PADDING)
      .ok_or(anyhow::anyhow!("width too small"))?;

    let fifo = tempdir.path().join("kak-popup-commands");
    unistd::mkfifo(&fifo, Mode::S_IRUSR | Mode::S_IWUSR)?;

    println!("{}", fifo.to_str().expect("fifo to_str"));

    Ok(Self {
      tmux: Tmux::new(&command, height, width)?,
      kakoune,
      cleanup,

      title,
      quit: AtomicBool::new(false),
      height: AtomicUsize::new(height),
      width: AtomicUsize::new(width),

      tempdir,
      fifo,
    })
  }

  fn wrap_command(cleanup: &Cleanup, command: &str, args: &[String]) -> String {
    let args: Vec<String> = args.iter().map(escape::bash).collect();
    let stdout = escape::bash(cleanup.stdout.to_string_lossy());
    let command = escape::bash(format!("{command} {} >{stdout}", args.join(" ")));

    // let stderr = &cleanup.stderr;

    format!("bash -c {command}")
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

    let title = if let Some(title) = &self.title {
      let title = escape::kak(format!("{}: (<c-space> to exit)", title));
      format!("-title {title}")
    } else {
      String::new()
    };

    let output = escape::kak(output.join("\n"));

    self.kakoune.eval(format!("info -style modal {title} {output}")).await?;

    Ok(())
  }

  async fn send_key(&self, key: &str) -> Result<()> {
    let mut key = match key {
      "<plus>" => "+",
      "<minut>" => "-",
      "<percent>" => "%",
      "<semicolon>" => ";",
      "<up>" => "Up",
      "<down>" => "Down",
      "<left>" => "Left",
      "<right>" => "Right",
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

      if event == "quit" {
        self.quit.store(true, Ordering::Relaxed);
        return Ok(());
      }

      if event.starts_with("resize") {
        match event.split(' ').collect::<Vec<&str>>().as_slice() {
          [_, height, width] => {
            let height = height
              .parse::<usize>()
              .context("height")?
              .checked_sub(Self::PADDING)
              .ok_or(anyhow::anyhow!("height too small"))?;

            let width = width
              .parse::<usize>()
              .context("width")?
              .checked_sub(Self::PADDING)
              .ok_or(anyhow::anyhow!("width too small"))?;

            self.height.store(height, Ordering::Relaxed);
            self.width.store(width, Ordering::Relaxed);

            self.tmux.resize_window(height, width).await?;
          }
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
    let refresh_loop = self.refresh_loop();
    let event_loop = self.event_loop();

    // TODO: find a way such that, we tell kakoune to cancel the modal and on-key
    //       as soon as either of the futures return, however we still want to
    //       wait for everything here and report errors (if any).
    if let Err(err) = tokio::try_join!(refresh_loop, event_loop) {
      self.kakoune.debug(format!("error: {err:?}")).await?;
    }

    self.kakoune.eval("popup-close").await?;

    Ok(())
  }

  #[tokio::main]
  async fn cleanup(&self) -> Result<()> {
    // TODO: if they passed in a kak_script, we need to call the command along with stdout, stderr, status etc
    //       cleanup tempdir should be automatic
    if let Some(Cleanup {
      kak_script,
      stdout,
      stderr,
    }) = &self.cleanup
    {
      let kak_script = escape::kak(kak_script);
      let stdout = escape::kak(tokio_fs::read_to_string(stdout).await?.trim());
      // let stderr = tokio_fs::read_to_string(stderr).await?.replace('\'', "''");

      self
        .kakoune
        .eval(format!("popup-handle-output {stdout} '' {kak_script}"))
        .await?;
    }

    Ok(())
  }

  pub fn start(&self) -> Result<()> {
    self.daemonize()?;
    self.run()?;
    self.cleanup()?;

    Ok(())
  }
}

impl Drop for Popup {
  fn drop(&mut self) {
    if let Err(err) = self.tmux.kill() {
      self
        .kakoune
        .sync_debug(format!("failed to kill tmux session {}: {err:?}", self.tmux.session))
        .expect("debug");
    }

    self.kakoune.sync_debug("exiting popup").expect("debug");
  }
}
