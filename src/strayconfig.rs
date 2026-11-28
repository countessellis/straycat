use std::{
  fmt,
};
use url::Url;

use crate::collections::Config;

use crate::defaults::*;

///////////// StrayWebSocketConfig

pub struct StrayConfig {
  pub target:  String,
  pub url:     String,
  pub port:    u16,
  pub source:  Option<String>,
  pub headers: Vec<(String, String)>,
  pub binary:  bool,
  pub crlf:    bool,
  pub verbose: bool,
  pub pretty:  bool,
  pub secure:  bool,
}

impl fmt::Display for StrayConfig {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let header_width: usize = self.headers.iter().map(|(h,_)| h.len()).max().unwrap_or(0) + 5;
    let headers: String = self.headers.iter().map(|(header,value)| { let header = format!("{}:",header); format!("{:<width$} {}",header,value, width=header_width) }).collect::<Vec<String>>().join("\n");
    write!(f,
"
  -----------------------------------
            Stray Config 
  -----------------------------------

    Target:  {}
    URL:     {}
    Port:    {}
    Source:  {}
    Binary:  {}
    CRLF:    {}
    Verbose: {}
    Pretty:  {}
    Secure:  {}

    Headers:

{}

  -----------------------------------

",
  self.target,
  self.url,
  self.port,
  match self.source { Some(ref source) => source.as_str(), None => "Default" },
  self.binary,
  self.crlf,
  self.verbose,
  self.pretty,
  self.secure,
  headers,
)
  }
}


impl StrayConfig {
  pub fn from_config(config: &Config) -> Self {
    log::info!("Building a config to catch a stray cat.");
    Self {
      target: {
        let target = config.get("target");
        if target.contains("://") {
          match Url::parse(&target) {
            Ok(url) => url.host_str().ok_or(target.clone()).map_or_else(|_| target.clone(), |target| target.trim_matches(['[', ']']).to_string()),
            Err(_)  => target,
          }
        } else {
          let host = match target.rsplit_once(':') {
            Some((host,port)) if !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) => host,
            _ => target.as_str(),
          };
          host.trim_matches(['[', ']']).to_string()
        }
      },
      url: config.get("target"),
      port: {
        match config.get("target").as_str() {
          target if let Some((_,port)) = target.rsplit_once(':') && !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) => {
            port.parse::<u16>().unwrap_or(0)
          },
          target if target.starts_with("http://")|target.starts_with("ws://") => 80,
          target if target.starts_with("https://")|target.starts_with("wss://") => 443,
          _ => {
            let port: String = config.get("port");
            port.parse::<u16>().unwrap_or(0)
          },
        }
      },
      source: {
        let mut addr: String = config.get("source_addr");
        let port: String = config.get("source_port");
        if addr.contains("::") { addr = format!("[{}]",addr); }
        match (addr.is_empty(),port.is_empty()) {
          (false,false) => Some(format!("{}:{}",addr,port)),
          (true,false) => Some(format!("0.0.0.0:{}",port)),
          (false,true) => Some(format!("{}:0",addr)),
          _            => None,
        }
      },
      headers: Vec::new(),
      binary: false,
      crlf: {
        let crlf: String = config.get("crlf");
        crlf.to_lowercase().parse::<bool>().unwrap_or(DEFAULT_STRAY_CRLF)
      },
      verbose: {
        let verbose: String = config.get("stray_verbose");
        verbose.to_lowercase().parse::<bool>().unwrap_or(DEFAULT_STRAY_VERBOSE)
      },
      pretty: {
        let verbose: String = config.get("pretty");
        verbose.to_lowercase().parse::<bool>().unwrap_or(DEFAULT_STRAY_PRETTY)
      },
      secure: {
        let secure = Some(config.get("tls")).filter(|tls| !tls.is_empty())
          .or_else(|| Some(config.get("secure")).filter(|secure| !secure.is_empty()))
          .map(|secure| secure.to_lowercase().parse::<bool>().unwrap_or(DEFAULT_STRAY_SECURE)).unwrap_or(false);
        match config.get("target").as_str() {
          target if target.starts_with("http://")|target.starts_with("ws://") => false,
          target if target.starts_with("https://")|target.starts_with("wss://") => true,
          _ => {
            secure
          },
        }
      },
    }
  }
}

