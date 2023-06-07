mod ansi;
mod style;

use std::str;

use anyhow::Result;

use self::{ansi::EscapeStack, style::Style};
use crate::tmux::DisplayInfo;

pub struct Buffer {
  info: DisplayInfo,
  data: Vec<Vec<u8>>,
}

impl Buffer {
  pub fn new(info: DisplayInfo, data: Vec<u8>) -> Self {
    let mut lines = Vec::new();
    let mut line = Vec::new();

    for byte in data {
      if byte == b'\n' {
        lines.push(line);
        line = Vec::new();
      } else {
        line.push(byte);
      }
    }

    if !line.is_empty() {
      lines.push(line);
    }

    Self { info, data: lines }
  }

  pub fn markup(self) -> Result<String> {
    let mut markup = String::new();
    let mut esc = EscapeStack::new();
    let mut style: Style = Style::default();

    for (y, line) in self.data.into_iter().enumerate() {
      let mut x: usize = 0;
      let mut fill = false;
      let mut chars = str::from_utf8(&line)?.chars();

      while x < self.info.size.width {
        let c = if fill {
          ' '
        } else if let Some(c) = chars.next() {
          c
        } else {
          markup.push_str(&Style::default().markup());
          fill = true;
          ' '
        };

        let (skip, new_style) = esc.skip(c);
        if let Some(new_style) = new_style {
          style.merge(&new_style);
          markup.push_str(&style.markup());
        }
        if skip {
          continue;
        }

        let at_cursor = x == self.info.cursor.x && y == self.info.cursor.y;

        if at_cursor {
          markup.push_str("{PrimaryCursor}");
        }

        match c {
          '{' => markup.push_str("\\{"),
          '\\' => markup.push_str("\\\\"),
          c => markup.push(c),
        }

        if at_cursor && fill {
          markup.push_str(&Style::default().markup());
        } else if at_cursor {
          markup.push_str(&style.markup());
        }

        x += 1;
      }

      // continue any styling preceding the inserted spaces
      if fill {
        markup.push_str(&style.markup());
      }

      markup.push('\n');
    }

    // don't include last newline
    markup.pop();

    Ok(markup)
  }
}
