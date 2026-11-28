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
      ("-4".to_string(),ConfigFlag::new("stray_ip_version",false,Some("4"))),
      ("-6".to_string(),ConfigFlag::new("stray_ip_version",false,Some("6"))),
      ("-C".to_string(),ConfigFlag::new("crlf",false,Some("true"))),
      ("-e".to_string(),ConfigFlag::new("stray_exec",true,None)),
      ("-l".to_string(),ConfigFlag::new("stray_listen",false,Some("true"))),
      ("-p".to_string(),ConfigFlag::new("source_port",true,None)),
      ("-s".to_string(),ConfigFlag::new("source_addr",true,None)),
      ("-U".to_string(),ConfigFlag::new("stray_unix_socket",true,None)),
      ("-v".to_string(),ConfigFlag::new("stray_verbose",false,Some("true"))),
    ]),
  };
  configurator.to_config()
}
