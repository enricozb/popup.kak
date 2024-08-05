use std::str::FromStr;

use anyhow::Result;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Point {
  pub x: usize,
  pub y: usize,
}

/// Parses a point from a `line.column` string (y first then x).
impl FromStr for Point {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self> {
    let parts: Vec<_> = s.split('.').collect();

    anyhow::ensure!(parts.len() == 2);

    Ok(Self {
      x: parts[1].parse()?,
      y: parts[0].parse()?,
    })
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct Size {
  pub height: usize,
  pub width: usize,
}

impl Size {
  pub fn padded(self, padding: usize) -> Result<Self> {
    Ok(Self {
      height: self
        .height
        .checked_sub(padding)
        .ok_or(anyhow::anyhow!("height too small"))?,

      width: self
        .width
        .checked_sub(padding)
        .ok_or(anyhow::anyhow!("width too small"))?,
    })
  }
}
