use std::{iter, str};

use anyhow::Result;

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
        if line.len() < info.size.width {
          let padding = info.size.width - line.len();
          line.extend(iter::repeat(b' ').take(padding));
        }

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

    for (y, line) in self.data.into_iter().enumerate() {
      for (x, c) in str::from_utf8(&line)?.chars().enumerate() {
        let at_cursor = x == self.info.cursor.x && y == self.info.cursor.y;

        if at_cursor {
          markup.push_str("{PrimaryCursor}");
        }

        match c {
          '{' => markup.push_str("\\{"),
          '\\' => markup.push_str("\\\\"),
          c => markup.push(c),
        }

        if at_cursor {
          markup.push_str("{Default}");
        }
      }

      markup.push('\n');
    }

    // don't include last newline
    markup.pop();

    Ok(markup)
  }
}
