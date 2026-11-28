use std::{
  io::{self,IsTerminal,Read,Write},
};
use url::Url;

use crate::strayconfig::StrayConfig;
use crate::straystream::StrayStream;

///////////// StrayWebServer

pub struct StrayWebServer;

impl StrayWebServer {
  pub(crate) fn run(cfg: &StrayConfig) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Capturing a stray webserver at {}.",cfg.target);
    let url = Url::parse(&cfg.url)?;
    let path = match url.query() {
      Some(query) => format!("{}?{}",url.path(),query),
      None => url.path().to_string(),
    };
    let host_header = match url.port() {
      Some(port) => format!("{}:{}",cfg.target,port),
      None => cfg.target.clone(),
    };
    let mut body = Vec::new();
    if !io::stdin().is_terminal() { io::stdin().read_to_end(&mut body)?; }
    let method = if body.is_empty() { "GET" } else { "POST" };
    let mut stream = StrayStream::connect(cfg)?;
    log::info!("Connected: {stream}");
    if cfg.verbose {
      eprintln!("Connected: {stream}");
    }
    let mut req = format!("{method} {path} HTTP/1.1\r\n");
    let has_host = cfg.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("host"));
    if !has_host { req.push_str(&format!("Host: {host_header}\r\n")); }
    let has_ua = cfg.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("user-agent"));
    if !has_ua { req.push_str("User-Agent: StrayCat/0.1.0\r\n"); }
    let has_conn = cfg.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("connection"));
    if !has_conn { req.push_str("Connection: close\r\n"); }
    for (key,value) in &cfg.headers {
      req.push_str(&format!("{key}: {value}\r\n"));
    }
    if !body.is_empty() { req.push_str(&format!("Content-Length: {}\r\n", body.len())); }
    req.push_str("\r\n");
    if cfg.verbose {
      for line in req.lines() {
        eprintln!("> {line}");
      }
    }
    stream.write_all(req.as_bytes())?;
    if !body.is_empty() { stream.write_all(&body)?; }
    stream.flush()?;
    let mut raw_response = Vec::new();
    stream.read_to_end(&mut raw_response)?;
    let header_end = raw_response.windows(4).position(|w| w == b"\r\n\r\n").ok_or("Invalid HTTP response: missing header terminator")?;
    let header_bytes = &raw_response[..header_end];
    let body_bytes = &raw_response[header_end + 4..];
    let header_str = String::from_utf8_lossy(header_bytes);
    let mut is_chunked = false;
    for (i,line) in header_str.lines().enumerate() {
      if cfg.verbose {
        if i == 0 {
          eprintln!("{line}");
        } else {
          eprintln!("< {line}");
        }
      }
      if let Some((key,value)) = line.split_once(':') && key.trim().eq_ignore_ascii_case("transfer-encoding") && value.to_ascii_lowercase().contains("chunked") { is_chunked = true; }
    }
    let body = if is_chunked {
      Self::decode_chunked(body_bytes)?
    } else {
      body_bytes.to_vec()
    };
    if cfg.pretty {
      if let Ok(text) = std::str::from_utf8(&body) {
        println!("{}", Self::format_json(text));
        return Ok(());
      }
    }
    let mut stdout = io::stdout().lock();
    stdout.write_all(&body)?;
    stdout.flush()?;
    Ok(())
  }

  fn decode_chunked(mut data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    while !data.is_empty() {
      let line_end = data.windows(2).position(|w| w == b"\r\n").ok_or("Malformed chunked encoding")?;
      let size_str = std::str::from_utf8(&data[..line_end])?.split(';').next().unwrap_or("").trim();
      let chunk_size = usize::from_str_radix(size_str, 16)?;
      data = &data[line_end + 2..];
      if chunk_size == 0 { break; }
      if data.len() < chunk_size + 2 { return Err("Unexpected EOF in chunked body".into()); }
      out.extend_from_slice(&data[..chunk_size]);
      data = &data[chunk_size + 2..];
    }
    Ok(out)
  }

  fn format_json(raw: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(raw) {
      Ok(val) => serde_json::to_string_pretty(&val).unwrap_or_else(|_| raw.to_string()),
      Err(_) => raw.to_string(),
    }
  }
}
