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

  pub fn skip(&mut self, c: char) -> (bool, Option<Style>) {
    let skip: bool;

    (self.state, skip) = match (&self.state, c) {
      (State::Wait, '\u{1b}') => (State::Esc, true),
      (State::Esc, '[') => (State::Sequence, true),
      (State::Sequence, 'm') => (State::Wait, true),

      (State::Wait | State::Esc, _) => (State::Wait, false),
      (State::Sequence, _) => (State::Sequence, true),
    };

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

      (skip, style)
    } else {
      (skip, None)
    }
  }
}
