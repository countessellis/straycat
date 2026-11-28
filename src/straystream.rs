use crossbeam_channel::unbounded;
use openssl::{
  ssl::{NameType,SslConnector,SslMethod,SslStream,SslVerifyMode},
  x509::{X509,X509VerifyResult},
};
use socket2::{Domain,Protocol,Socket,Type};
use std::{
  fmt,
  io::{self,ErrorKind,Read,Write},
  net::{SocketAddr,TcpStream,ToSocketAddrs},
  thread,
  time::Duration,
};

use crate::strayconfig::*;

///////////// StrayStream

pub enum StrayStream {
  Tcp(TcpStream),
  Tls(SslStream<TcpStream>),
}

impl StrayStream {
  pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
    match self {
      StrayStream::Tcp(stream) => stream.set_read_timeout(timeout),
      StrayStream::Tls(stream) => stream.get_ref().set_read_timeout(timeout),
    }
  }
}

impl Read for StrayStream {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    match self {
      StrayStream::Tcp(stream) => stream.read(buf),
      StrayStream::Tls(stream) => stream.read(buf),
    }
  }
}

impl Write for StrayStream {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match self {
      StrayStream::Tcp(stream) => stream.write(buf),
      StrayStream::Tls(stream) => stream.write(buf),
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    match self {
      StrayStream::Tcp(stream) => stream.flush(),
      StrayStream::Tls(stream) => stream.flush(),
    }
  }
}

impl fmt::Display for StrayStream {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let local = self.local_addr().map(|a| a.to_string()).unwrap_or_else(|_| "unknown".into());
    let peer = self.peer_addr().map(|a| a.to_string()).unwrap_or_else(|_| "unknown".into());
    match self {
      StrayStream::Tcp(_) => write!(f, "TCP ({local} -> {peer})"),
      StrayStream::Tls(s) => {
        let ssl = s.ssl();
        let version = ssl.version_str();
        let cipher = ssl.current_cipher().map(|c| c.name()).unwrap_or("unknown");
        write!(f, "TLS ({local} -> {peer}) [{version}, {cipher}")?;
        if let Some(sni) = ssl.servername(NameType::HOST_NAME) {
          write!(f, ", sni={sni}")?;
        }
        if let Some(alpn) = ssl.selected_alpn_protocol() {
          write!(f, ", alpn={}", String::from_utf8_lossy(alpn))?;
        }
        write!(f, "]")
      }
    }
  }
}

impl StrayStream {
  pub fn connect(cfg: &StrayConfig) -> Result<StrayStream,Box<dyn std::error::Error>> {
    log::info!("Capturing a stray stream at {}.",cfg.target);
    let tcp_stream = if let Some(ref source) = cfg.source {
      let local_addr: SocketAddr = source.parse()?;
      let remote_addr = (cfg.target.clone(),cfg.port).to_socket_addrs()?.find(|addr| addr.is_ipv4() == local_addr.is_ipv4()).ok_or("No matching IPv4/IPv6 remote address found for source address")?;
      let domain = Domain::for_address(local_addr);
      let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
      socket.set_reuse_address(true)?;
      socket.bind(&local_addr.into())?;
      socket.connect(&remote_addr.into())?;
      socket.into()
    } else {
      TcpStream::connect((cfg.target.clone(),cfg.port))?
    };
    if cfg.secure {
      let mut builder = SslConnector::builder(SslMethod::tls())?;
      builder.set_verify(SslVerifyMode::NONE);
      let connector = builder.build();
      let mut ssl_config = connector.configure()?;
      ssl_config.set_verify_hostname(false);
      Ok(StrayStream::Tls(ssl_config.connect(&cfg.target,tcp_stream)?))
    } else {
      Ok(StrayStream::Tcp(tcp_stream))
    }
  }

  pub(crate) fn run(cfg: &StrayConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = Self::connect(cfg)?;
    log::info!("Connected: {}",stream);
    if cfg.verbose {
      eprintln!("Connected: {}",stream);
    }
    log::info!("Playing with a stray stream.");
    let _ = stream.set_nodelay(true);
    stream.set_read_timeout(Some(Duration::from_millis(20)))?;
    let (tx,rx) = unbounded::<Vec<u8>>();
    thread::spawn({
      let crlf = cfg.crlf;
      move || {
        let mut stdin = io::stdin().lock();
        let mut in_buf = [0u8; 4096];
        loop {
          match stdin.read(&mut in_buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
              let data = if crlf {
                let mut out = Vec::with_capacity(n + 16);
                for &b in &in_buf[..n] {
                  if b == b'\n' && out.last() != Some(&b'\r') {
                    out.push(b'\r');
                  }
                  out.push(b);
                }
                out
              } else {
                in_buf[..n].to_vec()
              };
              if tx.send(data).is_err() {
                break;
              }
            },
          }
        }
      }
    });
    let mut buf = [0u8; 4096];
    loop {
      while let Ok(bytes) = rx.try_recv() {
        stream.write_all(&bytes)?;
        stream.flush()?;
      }
      match stream.read(&mut buf) {
        Ok(0) => break,
        Ok(n) => {
          let mut stdout = io::stdout().lock();
          stdout.write_all(&buf[..n])?;
          stdout.flush()?;
        }
        Err(ref err) if err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut => {}
        Err(err) => return Err(err.into()),
      }
    }
    Ok(())
  }

  pub fn get_cert(&self) -> Option<X509> {
    match self {
      StrayStream::Tcp(_) => None,
      StrayStream::Tls(stream) => stream.ssl().peer_certificate(),
    }
  }

  pub fn verify_cert(&self) -> Option<X509VerifyResult> {
    match self {
      StrayStream::Tcp(_) => None,
      StrayStream::Tls(s) => Some(s.ssl().verify_result()),
    }
  }

  pub fn tcp_ref(&self) -> &TcpStream {
    match self {
      StrayStream::Tcp(stream) => stream,
      StrayStream::Tls(stream) => stream.get_ref(),
    }
  }

  pub fn peer_addr(&self) -> io::Result<SocketAddr> {
    self.tcp_ref().peer_addr()
  }

  pub fn local_addr(&self) -> io::Result<SocketAddr> {
    self.tcp_ref().local_addr()
  }

  pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
    self.tcp_ref().set_nodelay(nodelay)
  }

  pub fn is_tls(&self) -> bool {
    matches!(self,StrayStream::Tls(_))
  }
}
