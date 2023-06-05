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

  pub fn new(kakoune: Kakoune, title: Option<String>, height: usize, width: usize, command: &str) -> Result<Self> {
    Ok(Self {
      tmux: Tmux::new(command, height, width)?,
      kakoune,

      title,
      size: Arc::new(Mutex::new(Size { height, width }.padded(Self::PADDING)?)),

      keys_fifo: Fifo::new("keys")?,
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

    let keys_handle = Keys::new(
      self.kakoune.clone(),
      self.tmux.clone(),
      self.keys_fifo.clone(),
      self.commands_fifo.clone(),
      quit.clone(),
    )
    .spawn();

    let resize_handle = Resize::new(
      Self::PADDING,
      self.tmux.clone(),
      self.size.clone(),
      self.resize_fifo.clone(),
      quit.clone(),
    )
    .spawn();

    let refresh_handle = Refresh::new(
      self.kakoune.clone(),
      self.tmux.clone(),
      self.title.clone(),
      self.size.clone(),
      quit.clone(),
    )
    .spawn();

    quit.wait();

    self.hide()?;
    self.flush_fifos();

    Ok(())
  }

  fn hide(&self) -> Result<()> {
    println!("hiding...");

    self.kakoune.eval(
      "
        echo -debug 'hiding!!'
        execute-keys <c-space>
        info -style modal
        popup-unstyle-modal
        unset-option window popup_keys_fifo
        remove-hooks window popup
        echo -debug 'i left buddy'
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

    thread::sleep(std::time::Duration::from_secs(1));
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
