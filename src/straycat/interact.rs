use std::{
  fs,
  thread,
  time::Duration,
};

use crate::collections::Config;

use super::StrayCat;

use crate::defaults::*;

///////////// StrayCat

impl StrayCat {
  pub fn interact(&self) {
    loop {
      thread::sleep(Duration::from_secs(DEFAULT_WAIT));
    }
  }
}
