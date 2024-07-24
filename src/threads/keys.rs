use std::sync::mpsc::Sender;

use anyhow::Result;

use super::{Spawn, Step};
use crate::{fifo::Fifo, kakoune::Kakoune, tmux::Tmux};

pub struct Keys {
  tmux: Tmux,
  keys_fifo: Fifo,
  commands_fifo: Fifo,
  refresh: Sender<()>,
}

impl Keys {
  pub const QUIT_KEY: &'static str = "<c-space>";
  const CAPTURE_KEYS: &'static str = "popup-capture-keys";

  pub fn new(kakoune: &Kakoune, tmux: Tmux, keys_fifo: Fifo, commands_fifo: Fifo, refresh: Sender<()>) -> Result<Self> {
    kakoune.eval(Self::CAPTURE_KEYS)?;

    Ok(Self {
      tmux,
      keys_fifo,
      commands_fifo,
      refresh,
    })
  }
}

impl Spawn for Keys {
  const NAME: &'static str = "keys";

  fn step(&self) -> Result<Step> {
    let key = self.keys_fifo.read()?;
    let key = key.trim();

    if key == Self::QUIT_KEY {
      return Ok(Step::Quit);
    }

    if ! key.starts_with("<mouse:") {
      self.tmux.send_keys(&Key::from(key).into_tmux())?;
    }
    self.commands_fifo.write(Self::CAPTURE_KEYS)?;
    self.refresh.send(())?;

    Ok(Step::Next)
  }
}

struct Key<'a> {
  key: &'a str,

  alt: bool,
  ctrl: bool,
  shift: bool,
}

impl<'a> Key<'a> {
  fn into_tmux(self) -> String {
    // special key
    if self.key == "tab" && self.shift {
      return "BTab".to_string();
    }

    let mut tmux_key = String::new();
    if self.alt {
      tmux_key.push_str("M-");
    }
    if self.ctrl {
      tmux_key.push_str("C-");
    }
    if self.shift {
      tmux_key.push_str("S-");
    }

    tmux_key.push_str(match self.key {
      "lt" => "<",
      "gt" => ">",
      "plus" => "+",
      "minus" => "-",
      "percent" => "%",
      "semicolon" => "\\;",
      "up" => "Up",
      "down" => "Down",
      "left" => "Left",
      "right" => "Right",
      "esc" => "Escape",
      "ret" => "Enter",
      "tab" => "Tab",
      "space" => "Space",
      "backspace" => "BSpace",
      "del" => "DC",
      "quote" => "'",
      "dquote" => "\"",
      key => key,
    });

    tmux_key
  }
}

impl<'a> From<&'a str> for Key<'a> {
  fn from(key: &'a str) -> Self {
    let mut key = if key.starts_with('<') {
      &key[1..key.len() - 1]
    } else {
      key
    };

    let mut alt = false;
    let mut ctrl = false;
    let mut shift = false;

    while key.len() >= 3 {
      match &key[..2] {
        "a-" => alt = true,
        "c-" => ctrl = true,
        "s-" => shift = true,

        _ => break,
      }

      key = &key[2..];
    }

    Self { key, alt, ctrl, shift }
  }
}
