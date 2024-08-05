use std::{str::FromStr, sync::mpsc::Sender};

use anyhow::Result;

use super::{Spawn, Step};
use crate::{
  fifo::Fifo,
  geometry::Point,
  kakoune::Kakoune,
  tmux::{Key as TmuxKey, Tmux},
};

pub struct Keys {
  padding: usize,
  tmux: Tmux,
  keys_fifo: Fifo,
  commands_fifo: Fifo,
  refresh: Sender<()>,
}

impl Keys {
  pub const QUIT_KEY: &'static str = "<c-space>";
  const CAPTURE_KEYS: &'static str = "popup-capture-keys";

  pub fn new(
    kakoune: &Kakoune,
    padding: usize,
    tmux: Tmux,
    keys_fifo: Fifo,
    commands_fifo: Fifo,
    refresh: Sender<()>,
  ) -> Result<Self> {
    kakoune.eval(Self::CAPTURE_KEYS)?;

    Ok(Self {
      padding,
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

    let mut key = Key::try_from(key)?;
    key.unpad_coords(self.padding);

    self.tmux.send_keys(key.into())?;
    self.commands_fifo.write(Self::CAPTURE_KEYS)?;
    self.refresh.send(())?;

    Ok(Step::Next)
  }
}

struct Key<'a> {
  event: Event<'a>,
  modifiers: Modifiers,
}

impl<'a> Key<'a> {
  fn unpad_coords(&mut self, padding: usize) {
    match self.event {
      Event::Scroll {
        coords: Some(ref mut coords),
        ..
      }
      | Event::Mouse { ref mut coords, .. } => {
        coords.x = coords.x.saturating_sub(padding / 2).max(1);
        coords.y = coords.y.saturating_sub(padding / 2).max(1);
      }

      _ => (),
    }
  }
}

impl<'a> From<Key<'a>> for TmuxKey {
  fn from(key: Key<'a>) -> Self {
    let Key { event, modifiers } = key;

    match event {
      Event::Scroll { amount, coords } => {
        let up = amount > 0;
        // 64 is the scroll
        if let Some(Point { x: column, y: line }) = coords {
          let modifiers = 64 | usize::from(modifiers) | usize::from(up);

          TmuxKey::Esc(format!("<{modifiers};{column};{line}M"))
        } else if up {
          TmuxKey::Key("Up".to_string())
        } else {
          TmuxKey::Key("Down".to_string())
        }
      }

      Event::Mouse { button, action, coords } => {
        let modifiers = usize::from(modifiers) | button.map_or(0, usize::from) | usize::from(action);
        let Point { x: column, y: line } = coords;
        let release = if let MouseAction::Release = action { "m" } else { "M" };

        TmuxKey::Esc(format!("<{modifiers};{column};{line}{release}"))
      }

      Event::Named(s) => {
        // tmux's key names differs from kakoune's here, so this is a special case
        if s == "tab" && modifiers.shift {
          return TmuxKey::Key("BTab".into());
        }

        let mut tmux_key = String::new();

        if modifiers.alt {
          tmux_key.push_str("M-");
        }
        if modifiers.ctrl {
          tmux_key.push_str("C-");
        }
        if modifiers.shift {
          tmux_key.push_str("S-");
        }

        tmux_key.push_str(match s {
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
          key => key,
        });

        TmuxKey::Key(tmux_key)
      }
    }
  }
}

impl<'a> TryFrom<&'a str> for Key<'a> {
  type Error = anyhow::Error;

  fn try_from(key: &'a str) -> Result<Self> {
    let mut key = if key.starts_with('<') {
      &key[1..key.len() - 1]
    } else {
      return Ok(Self {
        event: Event::Named(key),
        modifiers: Modifiers::new(),
      });
    };

    let mut modifiers = Modifiers::new();

    while key.len() >= 3 {
      match &key[..2] {
        "a-" => modifiers.alt = true,
        "c-" => modifiers.ctrl = true,
        "s-" => modifiers.shift = true,

        _ => break,
      }

      key = &key[2..];
    }

    let event = if key.starts_with("mouse") {
      let parts: Vec<_> = key.split(':').collect();
      if parts[1] == "move" {
        Event::Mouse {
          action: MouseAction::Move,
          button: None,
          coords: parts[2].parse()?,
        }
      } else {
        Event::Mouse {
          action: parts[1].parse()?,
          button: Some(parts[2].parse()?),
          coords: parts[3].parse()?,
        }
      }
    } else if key.starts_with("scroll") {
      let parts: Vec<_> = key.split(':').collect();

      Event::Scroll {
        amount: parts[1].parse()?,
        coords: parts.get(2).and_then(|c| c.parse().ok()),
      }
    } else {
      Event::Named(key)
    };

    Ok(Self { event, modifiers })
  }
}

enum Event<'a> {
  Named(&'a str),

  Scroll {
    amount: i32,
    coords: Option<Point>,
  },

  Mouse {
    button: Option<MouseButton>,
    action: MouseAction,
    coords: Point,
  },
}

enum MouseButton {
  Left,
  Right,
}

impl From<MouseButton> for usize {
  fn from(button: MouseButton) -> usize {
    match button {
      MouseButton::Left => 0,
      MouseButton::Right => 2,
    }
  }
}

impl FromStr for MouseButton {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self> {
    match s {
      "left" => Ok(Self::Left),
      "right" => Ok(Self::Right),

      _ => anyhow::bail!("unknown mouse button {s:?}"),
    }
  }
}

#[derive(Clone, Copy)]
enum MouseAction {
  Move,
  Press,
  Release,
}

impl From<MouseAction> for usize {
  fn from(action: MouseAction) -> usize {
    match action {
      MouseAction::Move => 32,
      MouseAction::Press | MouseAction::Release => 0,
    }
  }
}

impl FromStr for MouseAction {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self> {
    match s {
      "move" => Ok(Self::Move),
      "press" => Ok(Self::Press),
      "release" => Ok(Self::Release),

      _ => anyhow::bail!("unknown mouse action {s:?}"),
    }
  }
}

#[derive(Default)]
struct Modifiers {
  alt: bool,
  ctrl: bool,
  shift: bool,
}

impl Modifiers {
  pub fn new() -> Self {
    Self::default()
  }
}

impl From<Modifiers> for usize {
  fn from(modifiers: Modifiers) -> usize {
    let alt = if modifiers.alt { 8 } else { 0 };
    let ctrl = if modifiers.ctrl { 16 } else { 0 };
    let shift = if modifiers.shift { 4 } else { 0 };

    alt | ctrl | shift
  }
}
