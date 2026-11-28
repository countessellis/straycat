use std::fmt;
use std::collections::HashMap;

use crate::util;

use crate::collections::{Config,Fields};

use crate::defaults::*;

///////////// Help

#[derive(Debug,Clone)]
pub(crate) struct Help {
  config: Config,
}

impl fmt::Display for Help {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let details: Vec<String> = vec![
      format!("  -4                                   Use IPv4 addresses only."),
      format!("  -6                                   Use IPv6 addresses only."),
      format!("  -C                                   Send CRLF as line-ending."),
      format!("  -l                                   Listen for an incoming connection rather than initiating a connection to a remote host."),
      format!("  -p <PORT>                            Specify the source port, 0-65536."),
      format!("  -s <ADDR>                            Specify the source IP, in IPv4 or IPv6."),
      format!("  -U <PATH>                            Specify a UNIX socket file instead of a network connection."),
      format!("  -v                                   Output verbose output for connection."),
      format!(""),
      format!("  --mode <MODE>                        Mode to run in (default: {}, choices: {})",DEFAULT_MODE,MODE_LIST.join(", ")),
      format!("  --target <MODE>                      Target to connect to, either a URL, a hostname, or a hostname:port pair."),
      format!("  --port <MODE>                        Target port to connect to."),
      format!("  --secure <true/false>                Use TLS to connect (default: {})",DEFAULT_STRAY_SECURE),
      format!("  --pretty <MODE>                      Enable pretty print of JSON coming from target (default: {})",DEFAULT_STRAY_PRETTY),
      format!(""),
      format!("  --verbose <true/false>               Enable or disable logging (default: {})",DEFAULT_LOG_VERBOSE),
      format!("  --debug <true/false>                 Enable or disable debug level logging (default: {})",DEFAULT_LOG_DEBUG),
      format!("  --help | -h                          Display this help message."),
      format!("  --version | -V                       Display version."),
    ];
    let fields: Fields = Fields::new(&HashMap::from([
      ("RUNAS",self.config.get("ranas")),
      ("DESCRIPTION",DESCRIPTION.to_string()),
      ("DETAILS",details.join("\n")),
      ("COPYRIGHT",COPYRIGHT.to_string()),
      ("AUTHORS",AUTHORS.to_string()),
    ]));
    let help: String = util::render(&HELP_TEMPLATE.to_string(),&fields);
    write!(f, "{}",help)
  }
}

impl Help {
  pub(crate) fn new(config: &Config) -> Help {
    Help { config: config.clone() }
  }
}

