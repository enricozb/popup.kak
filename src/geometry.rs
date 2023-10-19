use anyhow::Result;
use serde::Deserialize;

#[derive(Clone, Copy, Deserialize)]
pub struct Point {
  pub x: usize,
  pub y: usize,
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
