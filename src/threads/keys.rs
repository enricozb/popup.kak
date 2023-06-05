use anyhow::Result;

use super::{Spawn, Step};
use crate::{fifo::Fifo, kakoune::Kakoune, tmux::Tmux};

pub struct Keys {
  tmux: Tmux,
  keys_fifo: Fifo,
  commands_fifo: Fifo,
}

impl Keys {
  pub fn new(kakoune: &Kakoune, tmux: Tmux, keys_fifo: Fifo, commands_fifo: Fifo) -> Result<Self> {
    kakoune.eval("popup-capture-keys")?;

    Ok(Self {
      tmux,
      keys_fifo,
      commands_fifo,
    })
  }
}

impl Spawn for Keys {
  const NAME: &'static str = "keys";

  fn step(&self) -> Result<Step> {
    let key = self.keys_fifo.read()?;
    let key = key.trim();

    if key == "<c-space>" {
      println!("keys quitting!");
      return Ok(Step::Quit);
    }

    // TODO: trigger a refresh
    self.tmux.send_keys(&tmux_key(key))?;
    self.commands_fifo.write("popup-capture-keys")?;

    Ok(Step::Next)
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
