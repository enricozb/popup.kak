use anyhow::Result;

#[derive(Clone, Copy, Debug)]
pub enum Color {
  Black,
  Red,
  Green,
  Yellow,
  Blue,
  Magenta,
  Cyan,
  White,

  BrightBlack,
  BrightRed,
  BrightGreen,
  BrightYellow,
  BrightBlue,
  BrightMagenta,
  BrightCyan,
  BrightWhite,

  Rgb(u8, u8, u8),

  Default,
}

impl Color {
  pub fn from_256(val: u8) -> Self {
    match val {
      0..=7 => Color::from(val + 30),

      8..=15 => Color::from(val + 82),

      16..=231 => {
        let r = ((val - 16) / 36) * 51;
        let g = ((val - 16) % 36 / 6) * 51;
        let b = ((val - 16) % 6) * 51;

        Color::Rgb(r, g, b)
      }

      232..=255 => {
        let gray = (val - 232) * 10;

        Color::Rgb(gray, gray, gray)
      }
    }
  }

  pub fn markup(&self) -> String {
    match self {
      Self::Black => "black".to_string(),
      Self::Red => "red".to_string(),
      Self::Green => "green".to_string(),
      Self::Yellow => "yellow".to_string(),
      Self::Blue => "blue".to_string(),
      Self::Magenta => "magenta".to_string(),
      Self::Cyan => "cyan".to_string(),
      Self::White => "white".to_string(),

      Self::BrightBlack => "bright-black".to_string(),
      Self::BrightRed => "bright-red".to_string(),
      Self::BrightGreen => "bright-green".to_string(),
      Self::BrightYellow => "bright-yellow".to_string(),
      Self::BrightBlue => "bright-blue".to_string(),
      Self::BrightMagenta => "bright-magenta".to_string(),
      Self::BrightCyan => "bright-cyan".to_string(),
      Self::BrightWhite => "bright-white".to_string(),

      // TODO: need to convert to padded hex
      Self::Rgb(r, g, b) => format!("rgb:{r:02X}{g:02X}{b:02X}"),

      Self::Default => "default".to_string(),
    }
  }
}

impl From<u8> for Color {
  fn from(param: u8) -> Self {
    match param {
      30 => Self::Black,
      31 => Self::Red,
      32 => Self::Green,
      33 => Self::Yellow,
      34 => Self::Blue,
      35 => Self::Magenta,
      36 => Self::Cyan,
      37 => Self::White,

      39 => Self::Default,

      90 => Self::BrightBlack,
      91 => Self::BrightRed,
      92 => Self::BrightGreen,
      93 => Self::BrightYellow,
      94 => Self::BrightBlue,
      95 => Self::BrightMagenta,
      96 => Self::BrightCyan,
      97 => Self::BrightWhite,

      param => panic!("unexpected code: {param}"),
    }
  }
}

#[derive(Debug, Default)]
pub struct Style {
  foreground: Option<Color>,
  background: Option<Color>,
  bold: Option<bool>,
  dim: Option<bool>,
  italic: Option<bool>,
  underline: Option<bool>,
  blink: Option<bool>,
  reverse: Option<bool>,
  strike: Option<bool>,
}

impl Style {
  pub fn markup(&self) -> String {
    let foreground = self.foreground.as_ref().map(Color::markup).unwrap_or_default();
    let background = self.background.as_ref().map(Color::markup).unwrap_or_default();

    let mut attributes = String::new();

    if self.bold.unwrap_or_default() {
      attributes.push('b');
    };
    if self.dim.unwrap_or_default() {
      attributes.push('d');
    };
    if self.italic.unwrap_or_default() {
      attributes.push('i');
    };
    if self.underline.unwrap_or_default() {
      attributes.push('u');
    };
    if self.blink.unwrap_or_default() {
      attributes.push('B');
    };
    if self.reverse.unwrap_or_default() {
      attributes.push('r');
    };
    if self.strike.unwrap_or_default() {
      attributes.push('s');
    };

    format!("{{{foreground},{background}+{attributes}@Default}}")
  }

  pub fn merge(&mut self, other: &Self) {
    *self = Self {
      foreground: other.foreground.or(self.foreground),
      background: other.background.or(self.background),
      bold: other.bold.or(self.bold),
      dim: other.dim.or(self.dim),
      italic: other.italic.or(self.italic),
      underline: other.underline.or(self.underline),
      blink: other.blink.or(self.blink),
      reverse: other.reverse.or(self.reverse),
      strike: other.strike.or(self.strike),
    }
  }

  pub fn reset() -> Self {
    Self {
      foreground: Some(Color::Default),
      background: Some(Color::Default),
      bold: Some(false),
      dim: Some(false),
      italic: Some(false),
      underline: Some(false),
      blink: Some(false),
      reverse: Some(false),
      strike: Some(false),
    }
  }

  pub fn try_from(chars: &[char]) -> Result<Self> {
    let mut params = Vec::<u8>::new();
    let mut param = String::new();

    // first two characters are ESC [
    for c in &chars[2..] {
      match c {
        ';' => {
          params.push(param.parse()?);
          param.clear();
        }
        'm' => {
          params.push(param.parse()?);
          break;
        }
        _ => param.push(*c),
      }
    }

    let mut style = Self::default();

    let mut i = 0;
    while i < params.len() {
      let param = params[i];

      match param {
        0 => style = Self::reset(),

        1 => style.bold = Some(true),
        2 => style.dim = Some(true),
        3 => style.italic = Some(true),
        4 => style.underline = Some(true),
        5 => style.blink = Some(true),
        7 => style.reverse = Some(true),
        9 => style.strike = Some(true),

        22 => {
          style.bold = Some(false);
          style.dim = Some(false);
        }
        23 => style.italic = Some(false),
        24 => style.underline = Some(false),
        25 => style.blink = Some(false),
        27 => style.reverse = Some(false),
        29 => style.strike = Some(false),

        38 if params[i + 1] == 2 => {
          style.foreground = Some(Color::Rgb(params[i + 2], params[i + 3], params[i + 4]));
          i += 5;
          continue;
        }

        38 if params[i + 1] == 5 => {
          style.foreground = Some(Color::from_256(params[i + 2]));
          i += 3;
          continue;
        }

        48 if params[i + 1] == 2 => {
          style.background = Some(Color::Rgb(params[i + 2], params[i + 3], params[i + 4]));
          i += 5;
          continue;
        }

        48 if params[i + 1] == 5 => {
          style.background = Some(Color::from_256(params[i + 2]));
          i += 3;
          continue;
        }

        38 | 48 => {
          return Err(anyhow::anyhow!(
            "unknown color mode {} in sequence: {params:?}",
            params[i + 1]
          ));
        }

        30..=39 | 90..=97 => style.foreground = Some(Color::from(param)),
        40..=49 | 100..=107 => style.background = Some(Color::from(param - 10)),

        param => println!("unknown param {param} in sequence: {params:?}, skipping"),
      }

      i += 1;
    }

    Ok(style)
  }
}
