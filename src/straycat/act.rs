use super::StrayCat;
use crate::straywebsocket::{StrayWebSocket,WsConfig};

use crate::defaults::*;

///////////// StrayCat

impl StrayCat {
  pub fn act(&self) {
    let target: String = self.config.get("target");
    match target.as_str() {
      target if target.starts_with("ws://")|target.starts_with("wss://") => {
        let config: WsConfig = WsConfig {
          url: target.to_string(),
          headers: Vec::new(),
          binary: false,
          verbose: {
            let verbose: String = self.config.get("stray_verbose");
            verbose.to_lowercase().parse::<bool>().unwrap_or(DEFAULT_STRAY_VERBOSE)
          },
          pretty: {
            let verbose: String = self.config.get("pretty");
            verbose.to_lowercase().parse::<bool>().unwrap_or(DEFAULT_STRAY_VERBOSE)
          },
        };
        match StrayWebSocket::run_ws(&config) {
          Ok(()) => {},
          Err(err) => log::error!("Failed to test websocket: {}",err),
        }
      },
      _ => {
      },
    }
  }
}
