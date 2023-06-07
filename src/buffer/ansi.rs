use anyhow::Result;

use super::style::Style;

#[derive(PartialEq, Eq)]
enum State {
  Wait,
  Esc,
  Sequence,
}

pub struct EscapeStack {
  state: State,
  sequence: Vec<char>,
}

impl EscapeStack {
  pub fn new() -> Self {
    Self {
      state: State::Wait,
      sequence: Vec::new(),
    }
  }

  pub fn skip(&mut self, c: char) -> Result<(bool, Option<Style>)> {
    let (new_state, skip) = match (&self.state, c) {
      (State::Wait, '\u{1b}') => (State::Esc, true),
      (State::Wait, _) => (State::Wait, false),

      (State::Esc, '[') => (State::Sequence, true),
      (State::Esc, _) => (State::Wait, false),

      (State::Sequence, 'm') => (State::Wait, true),
      (State::Sequence, _) => (State::Sequence, true),
    };

    self.state = new_state;

    if skip {
      self.sequence.push(c);
    }

    if self.state == State::Wait && !self.sequence.is_empty() {
      let style = match Style::try_from(self.sequence.as_slice()) {
        Ok(style) => Some(style),
        Err(err) => {
          println!("Style::try_from: {err:?}");
          None
        }
      };

      self.sequence.clear();

      Ok((skip, style))
    } else {
      Ok((skip, None))
    }
  }
}
