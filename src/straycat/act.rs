use super::StrayCat;

use crate::help::Help;
use crate::strayconfig::StrayConfig;
use crate::straystream::StrayStream;
use crate::straywebserver::StrayWebServer;
use crate::straywebsocket::StrayWebSocket;

///////////// StrayCat

impl StrayCat {
  pub fn act(&self) {
    let config: StrayConfig = StrayConfig::from_config(&self.config);
    log::debug!("{}",config);
    match config.url.as_str() {
      target if target.is_empty() => {
        let help: Help = Help::new(&self.config);
        println!("{}",help);
      },
      target if target.starts_with("ws://")|target.starts_with("wss://") => {
        match StrayWebSocket::run(&config) {
          Ok(()) => {},
          Err(err) => {
            log::error!("Failed to capture a stray websocket at {}: {}",config.url,err);
            eprintln!("Failed to capture a stray websocket at {}: {}",config.url,err);
          },
        }
      },
      target if target.starts_with("http://") || target.starts_with("https://") => {
        match StrayWebServer::run(&config) {
          Ok(()) => {},
          Err(err) => {
            log::error!("Failed to capture a stray webserver at {}: {}",config.url,err);
            eprintln!("Failed to capture a stray webserver at {}: {}",config.url,err);
          },
        }
      },
      target if target.contains("://") => {},
      _ => {
        match StrayStream::run(&config) {
          Ok(()) => {},
          Err(err) => {
            log::error!("Failed to capture a stray stream at {}:{}: {}",config.target,config.port,err);
            eprintln!("Failed to capture a stray stream at {}:{}: {}",config.target,config.port,err);
          },
        }
      },
    }
  }
}
