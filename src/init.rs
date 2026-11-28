use crate::collections::Config;
use crate::config;

use crate::help::*;
use crate::mode::*;
use crate::splash::*;
use crate::straycat::*;
use crate::version::*;

///////////// Run

pub fn run() {
  log::info!("{}",splash());
  log::info!("{}",version());
  let config: Config = config::init();
  log::info!("{}",config);
  let cat: StrayCat = StrayCat::new(&config);
  let mode: Mode = Mode::mode();
  match mode {
    Mode::Interactive => {
      println!("{}",splash());
      println!("{}",version());
      cat.interact();
    },
    Mode::Batch => {
      cat.act();
    },
    Mode::Help => {
      let help: Help = Help::new(&config);
      println!("{}",help);
    },
    Mode::Version => {
      println!("{}",version());
    },
  }
}
