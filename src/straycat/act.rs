use std::{
  fs,
};

use crate::collections::Config;

use super::StrayCat;
use super::websocket::*;

use crate::defaults::*;

///////////// StrayCat

impl StrayCat {
  pub fn act(&self) {
    let target: String = self.config.get("target");
    match target.as_str() {
      target if target.starts_with("ws://")|target.starts_with("wss://") => {
        let config: WsConfig = WsConfig { url: target.to_string(), headers: Vec::new(), binary: false, verbose: true };
        match Self::run_ws(&config) {
          Ok(()) => {},
          Err(err) => log::error!("Failed to test websocket: {}",err),
        }
      },
      _ => {
      },
    }
  }
}
