use std::{
  io::{self, BufRead, ErrorKind, Write},
  sync::mpsc,
  thread,
  time::Duration,
};
use tungstenite::{
  client,
  client::IntoClientRequest,
  http::header::{HeaderName, HeaderValue},
  Error as WsError,
  Message,
  WebSocket,
};

use crate::straystream::StrayStream;

pub struct WsConfig {
  pub url:     String,
  pub headers: Vec<(String, String)>,
  pub binary:  bool,
  pub verbose: bool,
  pub pretty:  bool,
}

pub struct StrayWebSocket;

impl StrayWebSocket {
  pub(crate) fn connect_ws(cfg: &WsConfig) -> Result<WebSocket<StrayStream>, Box<dyn std::error::Error>> {
    let mut request = cfg.url.as_str().into_client_request()?;
    for (key,value) in &cfg.headers {
      let name = HeaderName::from_bytes(key.as_bytes())?;
      let value = HeaderValue::from_str(value)?;
      request.headers_mut().append(name,value);
    }
    let uri = request.uri();
    let host = uri.host().ok_or("Missing host in WebSocket URL")?;
    let scheme = uri.scheme_str().unwrap_or("ws");
    let (port, secure) = match scheme {
      "ws"  => (uri.port_u16().unwrap_or(80), false),
      "wss" => (uri.port_u16().unwrap_or(443), true),
      other => return Err(format!("Unsupported URL scheme: {other}").into()),
    };
    let target = host.trim_matches(&['[', ']'][..]);
    let stream = StrayStream::connect(target,port,secure)?;
    log::info!("Connected: {stream}");
    if cfg.verbose {
      eprintln!("Connected: {stream}");
    }
    let (ws,response) = client(request,stream)?;
    if cfg.verbose {
      eprintln!("HTTP/1.1 {}",response.status());
      for (key,value) in response.headers() {
        eprintln!("{}: {}",key,value.to_str().unwrap_or("<binary>"));
      }
    }
    Ok(ws)
  }
  
  pub(crate) fn run_ws(cfg: &WsConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = Self::connect_ws(&cfg)?;
    socket.get_ref().set_read_timeout(Some(Duration::from_millis(20)))?;
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    thread::spawn(move || {
      let stdin = io::stdin();
      for line in stdin.lock().lines() {
        match line {
          Ok(text) => {
            if tx.send(text.into_bytes()).is_err() { break; }
          },
          Err(_) => break,
        }
      }
    });
    loop {
      while let Ok(bytes) = rx.try_recv() {
        let msg = if cfg.binary {
          Message::Binary(bytes.into())
        } else {
          Message::Text(String::from_utf8_lossy(&bytes).into_owned().into())
        };
        socket.send(msg)?;
      }
      match socket.read() {
        Ok(Message::Text(text)) => {
          if cfg.pretty {
            println!("{}",Self::format_json_stream(&text));
          } else {
            println!("{text}");
          }
        },
        Ok(Message::Binary(data)) => {
          let mut stdout = io::stdout().lock();
          stdout.write_all(&data)?;
          stdout.flush()?;
        },
        Ok(Message::Close(_)) => break,
        Ok(_) => {},
        Err(WsError::Io(ref e)) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {},
        Err(e) => return Err(e.into()),
      }
    }
    Ok(())
  }

  fn format_json_stream(raw: &str) -> String {
    let stream = serde_json::Deserializer::from_str(raw).into_iter::<serde_json::Value>();
    let mut out = Vec::new();
    for item in stream {
      match item {
        Ok(val) => out.push(serde_json::to_string_pretty(&val).unwrap_or_default()),
        Err(_) => return raw.to_string(),
      }
    }
    if out.is_empty() { raw.to_string() } else { out.join("\n") }
  }
}
