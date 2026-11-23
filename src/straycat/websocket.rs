use openssl::ssl::SslStream;
use std::{
  io::{self, BufRead, ErrorKind, Read, Write},
  net::TcpStream,
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

use super::StrayCat;

pub struct WsConfig {
  pub url: String,
  pub headers: Vec<(String, String)>,
  pub binary: bool,
  pub verbose: bool,
}

pub enum WsStream {
  Plain(TcpStream),
  Tls(SslStream<TcpStream>),
}

impl WsStream {
  fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
    match self {
      WsStream::Plain(stream) => stream.set_read_timeout(timeout),
      WsStream::Tls(stream) => stream.get_ref().set_read_timeout(timeout),
    }
  }
}

impl Read for WsStream {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    match self {
      WsStream::Plain(stream) => stream.read(buf),
      WsStream::Tls(stream) => stream.read(buf),
    }
  }
}

impl Write for WsStream {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match self {
      WsStream::Plain(stream) => stream.write(buf),
      WsStream::Tls(stream) => stream.write(buf),
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    match self {
      WsStream::Plain(stream) => stream.flush(),
      WsStream::Tls(stream) => stream.flush(),
    }
  }
}

impl StrayCat {
  pub(super) fn connect_ws(cfg: &WsConfig) -> Result<WebSocket<WsStream>, Box<dyn std::error::Error>> {
    let mut request = cfg.url.as_str().into_client_request()?;
    for (key,value) in &cfg.headers {
      let name = HeaderName::from_bytes(key.as_bytes())?;
      let value = HeaderValue::from_str(value)?;
      request.headers_mut().append(name,value);
    }
    let uri = request.uri();
    let host = uri.host().ok_or("Missing host in WebSocket URL")?;
    let scheme = uri.scheme_str().unwrap_or("ws");
    let stream = match scheme {
      "ws" => {
        let port = uri.port_u16().unwrap_or(80);
        let tcp = TcpStream::connect((host, port))?;
        WsStream::Plain(tcp)
      },
      "wss" => {
        return Err("wss:// TLS support is not implemented yet".into());
      },
      other => {
        return Err(format!("Unsupported URL scheme: {other}").into());
      },
    };
    let (ws, response) = client(request, stream)?;
    if cfg.verbose {
      eprintln!("HTTP/1.1 {}",response.status());
      for (key,value) in response.headers() {
        eprintln!("{}: {}",key,value.to_str().unwrap_or("<binary>"));
      }
    }
    Ok(ws)
  }
  
  pub(super) fn run_ws(cfg: &WsConfig) -> Result<(), Box<dyn std::error::Error>> {
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
        Ok(Message::Text(text)) => println!("{text}"),
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
}
