use std::collections::HashMap;

use crate::util;

use crate::collections::Config;

use crate::defaults::*;

///////////// Config

pub(crate) fn init() -> Config {
  let mut config = Config::from_hashmap(&HashMap::from([
    ("ranas".to_string(),util::bin_name()),
  ]));
  config.from_args();
  config
}
