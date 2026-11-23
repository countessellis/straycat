use std::{
  fmt,
  ops::Deref,
};

///////////// Sequence

#[derive(Debug, Clone)]
pub struct Sequence {
  pub sequence: u64,
}

impl Default for Sequence {
  fn default() -> Self {
    Self { sequence: 0, }
  }
}

impl fmt::Display for Sequence {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f,"{}",self.sequence)
  }
}

impl Deref for Sequence {
  type Target = u64;
  fn deref(&self) -> &Self::Target {
    &self.sequence
  }
}

impl Sequence {
  pub fn next(&mut self) -> u64 {
    self.sequence = self.sequence.saturating_add(1);
    if self.sequence == u64::MAX {
      log::error!("Maximum sequence reached!");
    }
    self.sequence
  }

  pub fn update(&mut self, seq: u64) -> u64 {
    self.sequence = seq;
    self.sequence
  }
}
