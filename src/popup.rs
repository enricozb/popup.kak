use std::{sync::Arc, thread};

use anyhow::Result;
use parking_lot::Mutex;

use crate::{
  fifo::Fifo,
  geometry::Size,
  kakoune::Kakoune,
  threads::{Keys, Quit, Refresh, Resize, Spawn},
  tmux::Tmux,
};

pub struct Popup {
  tmux: Tmux,
  kakoune: Kakoune,

  title: Option<String>,
  size: Arc<Mutex<Size>>,

  keys_fifo: Fifo,
  resize_fifo: Fifo,
  commands_fifo: Fifo,
}

impl Popup {
  const PADDING: usize = 16;

  pub fn new(
    kakoune: Kakoune,
    keys_fifo: Fifo,
    title: Option<String>,
    height: usize,
    width: usize,
    command: &str,
  ) -> Result<Self> {
    let size = Size { height, width }.padded(Self::PADDING)?;

    Ok(Self {
      tmux: Tmux::new(command, size)?,
      kakoune,

      title,
      size: Arc::new(Mutex::new(size)),

      keys_fifo,
      resize_fifo: Fifo::new("resize")?,
      commands_fifo: Fifo::new("commands")?,
    })
  }

  fn set_options(&self) -> Result<()> {
    self.kakoune.eval(format!(
      "
        set-option window popup_keys_fifo {keys_fifo}
        set-option window popup_resize_fifo {resize_fifo}
        set-option window popup_commands_fifo {commands_fifo}
      ",
      keys_fifo = self.keys_fifo.path_str()?,
      resize_fifo = self.resize_fifo.path_str()?,
      commands_fifo = self.commands_fifo.path_str()?,
    ))?;

    Ok(())
  }

  fn set_resize_hook(&self) -> Result<()> {
    self.kakoune.eval(
      r#"
        hook -group popup window WinResize .* %{
          echo -to-file %opt{popup_resize_fifo} "{
            ""height"": %val{window_height},
            ""width"": %val{window_width}
          }"
        }
      "#,
    )?;

    Ok(())
  }

  pub fn show(&self) -> Result<()> {
    self.set_options()?;
    self.set_resize_hook()?;
    self.kakoune.eval("popup-style-modal")?;

    let quit = Quit::new();

    let refresh = Refresh::new(
      self.kakoune.clone(),
      self.tmux.clone(),
      self.title.clone(),
      self.size.clone(),
    );

    let keys = Keys::new(
      &self.kakoune,
      self.tmux.clone(),
      self.keys_fifo.clone(),
      self.commands_fifo.clone(),
      refresh.sender.clone(),
    )?;

    let resize = Resize::new(
      Self::PADDING,
      self.tmux.clone(),
      self.size.clone(),
      self.resize_fifo.clone(),
      refresh.sender.clone(),
    );

    keys.spawn(self.kakoune.clone(), quit.clone());
    resize.spawn(self.kakoune.clone(), quit.clone());
    refresh.spawn(self.kakoune.clone(), quit.clone());

    self.kakoune.debug("waiting for quit")?;

    quit.wait();

    self.kakoune.debug("done waiting")?;

    self.hide()?;
    self.flush_fifos();

    Ok(())
  }

  fn hide(&self) -> Result<()> {
    self.kakoune.eval(
      "
        execute-keys <c-space>
        info -style modal
        popup-unstyle-modal
        unset-option window popup_keys_fifo
        remove-hooks window popup
      ",
    )?;

    Ok(())
  }

  fn flush_fifos(&self) {
    let keys_fifo = self.keys_fifo.clone();
    let resize_fifo = self.resize_fifo.clone();
    let commands_fifo = self.commands_fifo.clone();

    thread::spawn(move || keys_fifo.read());
    thread::spawn(move || resize_fifo.read());
    thread::spawn(move || commands_fifo.write("nop"));
  }
}

impl Drop for Popup {
  fn drop(&mut self) {
    if let Err(err) = self.tmux.kill() {
      self
        .kakoune
        .debug(format!("failed to kill tmux session {}: {err:?}", self.tmux.session))
        .expect("debug");
    }

    self.kakoune.debug("exiting popup").expect("debug");
  }
}
