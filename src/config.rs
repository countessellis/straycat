use std::collections::HashMap;

use crate::util;

use crate::collections::{Config,ConfigFlag,Configurator};

use crate::defaults::*;

///////////// Config

pub(crate) fn init() -> Config {
  let configurator = Configurator {
    defaults: HashMap::from([
      ("ranas".to_string(),util::bin_name()),
      ("pretty".to_string(),DEFAULT_STRAY_PRETTY.to_string()),
      ("secure".to_string(),DEFAULT_STRAY_SECURE.to_string()),
    ]),
    file: None,
    ignore: vec![
      "--interactive".to_string(),
      "--help".to_string(),
      "-h".to_string(),
      "--version".to_string(),
      "-V".to_string(),
    ],
    arguments: vec![
      "target".to_string(),
      "port".to_string(),
    ],
    flags: HashMap::from([
      ("-C".to_string(),ConfigFlag::new("crlf",false,Some("true"))),
      ("-p".to_string(),ConfigFlag::new("source_port",true,None)),
      ("-s".to_string(),ConfigFlag::new("source_addr",true,None)),
      ("-v".to_string(),ConfigFlag::new("stray_verbose",false,Some("true"))),
    ]),
  };
  configurator.to_config()
}
