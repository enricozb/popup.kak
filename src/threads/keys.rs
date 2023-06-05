use anyhow::Result;

use super::Spawn;
use crate::{fifo::Fifo, kakoune::Kakoune, threads::Quit, tmux::Tmux};

pub struct Keys {
  kakoune: Kakoune,
  tmux: Tmux,
  keys_fifo: Fifo,
  commands_fifo: Fifo,
  quit: Quit,
}

impl Keys {
  pub fn new(kakoune: Kakoune, tmux: Tmux, keys_fifo: Fifo, commands_fifo: Fifo, quit: Quit) -> Self {
    Self {
      kakoune,
      tmux,
      keys_fifo,
      commands_fifo,
      quit,
    }
  }
}

impl Spawn for Keys {
  fn run(&self) -> Result<()> {
    self.kakoune.eval("popup-capture-keys")?;

    while !self.quit.is_quit() {
      println!("keys");

      let key = self.keys_fifo.read()?;
      let key = key.trim();

      if key == "<c-space>" {
        self.quit.quit();
        return Ok(());
      }

      // TODO: trigger a refresh
      self.tmux.send_keys(&tmux_key(key))?;

      self.commands_fifo.write("popup-capture-keys")?;
    }

    Ok(())
  }
}

impl Drop for Keys {
  fn drop(&mut self) {
    self.quit.quit();
  }
}

fn tmux_key(key: &str) -> String {
  let key = match key {
    "<plus>" => "+",
    "<minus>" => "-",
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

  // TODO: handle <a-*> <s-*> and combinations <c-a-w>
  if key.starts_with("<c-") {
    format!("C-{}", &key[3..key.len() - 1])
  } else {
    key.to_string()
  }
}
