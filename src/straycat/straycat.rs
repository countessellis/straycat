use std::{
  fs,
  thread,
  time::Duration,
};

use crate::collections::Config;

use crate::defaults::*;

///////////// StrayCat

pub struct StrayCat {
  pub config: Config,
}

impl StrayCat {
  pub fn new(config: &Config) -> StrayCat {
    log::debug!("Initializing Stray Cat...");
    Self {
      config: config.clone(),
    }
  }
}
