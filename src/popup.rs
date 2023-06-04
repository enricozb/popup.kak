use std::{
  cell::Cell,
  sync::atomic::{AtomicBool, Ordering},
  time::Duration,
};

use anyhow::Result;
use tokio::time as tokio_time;

use crate::{escape, fifo::Fifo, geometry::Size, kakoune::Kakoune, tmux::Tmux};

pub struct Popup {
  tmux: Tmux,
  kakoune: Kakoune,

  title: Option<String>,
  size: Cell<Size>,

  keys_fifo: Fifo,
  resize_fifo: Fifo,
  commands_fifo: Fifo,

  quit: AtomicBool,
}

impl Popup {
  const PADDING: usize = 16;
  const REFRESH: Duration = Duration::from_millis(200);

  pub fn new(kakoune: Kakoune, title: Option<String>, height: usize, width: usize, command: &str) -> Result<Self> {
    Ok(Self {
      tmux: Tmux::new(command, height, width)?,
      kakoune,

      title,
      size: Cell::new(Size { height, width }.padded(Self::PADDING)?),

      keys_fifo: Fifo::new("keys")?,
      resize_fifo: Fifo::new("resize")?,
      commands_fifo: Fifo::new("commands")?,

      quit: AtomicBool::new(false),
    })
  }

  async fn refresh_loop(&self) -> Result<()> {
    loop {
      if self.quit.load(Ordering::Relaxed) {
        return Ok(());
      }

      self.refresh().await?;

      tokio_time::sleep(Self::REFRESH).await;
    }
  }

  async fn refresh(&self) -> Result<()> {
    let content = self.tmux.capture_pane().await?;

    let width = self.size.get().width;

    let output = String::from_utf8_lossy(&content[..content.len() - 1]);
    let mut output: Vec<&str> = output.rsplitn(2, '\n').collect();
    output.reverse();

    let last_line;
    if let Some(last) = output.last_mut() {
      last_line = format!("{last:<width$}");
      *last = &last_line;
    }

    let title = if let Some(title) = &self.title {
      let title = escape::kak(format!("{title}: (<c-space> to exit)"));
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

    // handle <a-*> <s-*> and combinations <c-a-w>
    let new_key;
    if key.starts_with("<c-") {
      new_key = format!("C-{}", &key[3..key.len() - 1]);
      key = &new_key;
    }

    self.tmux.send_keys(key).await?;
    self.refresh().await?;

    Ok(())
  }

  async fn key_loop(&self) -> Result<()> {
    self.kakoune.eval("popup-capture-keys").await?;

    loop {
      let key = self.keys_fifo.read().await?;
      let key = key.trim();

      if key == "<c-space>" {
        self.quit.store(true, Ordering::Relaxed);
        self.commands_fifo.write("nop").await?;
        return Ok(());
      }

      self.send_key(key).await?;
      self.commands_fifo.write("popup-capture-keys").await?;
    }
  }

  async fn resize_loop(&self) -> Result<()> {
    loop {
      let resize = self.resize_fifo.read().await?;
      let new_size: Size = serde_json::from_str(&resize)?;

      self.size.set(new_size);
      self.tmux.resize_window(new_size).await?;
    }
  }

  async fn set_options(&self) -> Result<()> {
    self
      .kakoune
      .eval(format!(
        "
          set-option window popup_keys_fifo {keys_fifo}
          set-option window popup_resize_fifo {resize_fifo}
          set-option window popup_commands_fifo {commands_fifo}
        ",
        keys_fifo = self.keys_fifo.path_str()?,
        resize_fifo = self.resize_fifo.path_str()?,
        commands_fifo = self.commands_fifo.path_str()?,
      ))
      .await?;

    Ok(())
  }

  async fn hook(&self) -> Result<()> {
    self
      .kakoune
      .eval(
        r#"
          hook -group popup window WinResize .* %{
            echo -to-file %opt{popup_resize_fifo} "{
              'height': %val{window_height},
              'width': %val{window_width}
            }"
          }
        "#,
      )
      .await?;

    Ok(())
  }

  #[tokio::main]
  pub async fn show(&self) -> Result<()> {
    self.set_options().await?;
    self.hook().await?;
    self.kakoune.eval("popup-style-modal").await?;

    tokio::select! {
      _ = self.key_loop() => {}
      _ = self.resize_loop() => {}
      _ = self.refresh_loop() => {}
    };

    self.hide().await?;

    Ok(())
  }

  async fn hide(&self) -> Result<()> {
    self
      .kakoune
      .eval(
        "
          execute-keys <c-space>
          info -style modal
          popup-unstyle-modal
          unset-option window popup_keys_fifo
          remove-hooks window popup
        ",
      )
      .await?;

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
