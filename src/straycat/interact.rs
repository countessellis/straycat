use std::{
  thread,
  time::Duration,
};

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
