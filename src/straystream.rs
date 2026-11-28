use crossbeam_channel::unbounded;
use openssl::{
  ssl::{NameType,SslConnector,SslMethod,SslStream,SslVerifyMode},
  x509::{X509,X509VerifyResult},
};
use std::{
  fmt,
  io::{self,ErrorKind,Read,Write},
  net::{SocketAddr,TcpStream},
  thread,
  time::Duration,
};
#[cfg(unix)]
use std::os::unix::net::UnixStream;

use crate::strayconfig::StrayConfig;
use crate::straysocket::StraySocket;

///////////// StrayStream

pub enum StrayStream {
  Tcp(TcpStream),
  Tls(SslStream<TcpStream>),
  #[cfg(unix)]
  Unix(UnixStream),
  #[cfg(unix)]
  UnixTls(SslStream<UnixStream>),
}

impl StrayStream {
  pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
    match self {
      StrayStream::Tcp(stream) => stream.set_read_timeout(timeout),
      StrayStream::Tls(stream) => stream.get_ref().set_read_timeout(timeout),
      #[cfg(unix)]
      StrayStream::Unix(s) => s.set_read_timeout(timeout),
      #[cfg(unix)]
      StrayStream::UnixTls(s) => s.get_ref().set_read_timeout(timeout),
    }
  }
}

impl Read for StrayStream {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    match self {
      StrayStream::Tcp(stream) => stream.read(buf),
      StrayStream::Tls(stream) => stream.read(buf),
      #[cfg(unix)]
      StrayStream::Unix(s) => s.read(buf),
      #[cfg(unix)]
      StrayStream::UnixTls(s) => s.read(buf),
    }
  }
}

impl Write for StrayStream {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match self {
      StrayStream::Tcp(stream) => stream.write(buf),
      StrayStream::Tls(stream) => stream.write(buf),
      #[cfg(unix)]
      StrayStream::Unix(s) => s.write(buf),
      #[cfg(unix)]
      StrayStream::UnixTls(s) => s.write(buf),
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    match self {
      StrayStream::Tcp(stream) => stream.flush(),
      StrayStream::Tls(stream) => stream.flush(),
      #[cfg(unix)]
      StrayStream::Unix(s) => s.flush(),
      #[cfg(unix)]
      StrayStream::UnixTls(s) => s.flush(),
    }
  }
}

impl fmt::Display for StrayStream {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      StrayStream::Tcp(_) | StrayStream::Tls(_) => {
        let local = self.local_addr().map(|a| a.to_string()).unwrap_or_else(|_| "unknown".into());
        let peer = self.peer_addr().map(|a| a.to_string()).unwrap_or_else(|_| "unknown".into());
        match self {
          StrayStream::Tcp(_) => write!(f, "TCP ({local} -> {peer})"),
          StrayStream::Tls(s) => Self::fmt_tls(f,&format!("TLS ({local} -> {peer})"), s.ssl()),
          _ => unreachable!(),
        }
      },
      #[cfg(unix)]
      StrayStream::Unix(s) => {
        let peer = s.peer_addr().ok().and_then(|a| a.as_pathname().map(|p| p.display().to_string())).unwrap_or_else(|| "unnamed".into());
        write!(f, "UNIX ({peer})")
      },
      #[cfg(unix)]
      StrayStream::UnixTls(s) => {
        let peer = s.get_ref().peer_addr().ok().and_then(|a| a.as_pathname().map(|p| p.display().to_string())) .unwrap_or_else(|| "unnamed".into());
        Self::fmt_tls(f, &format!("UNIX+TLS ({peer})"),s.ssl())
      },
    }
  }
}

impl StrayStream {
  pub fn connect(cfg: &StrayConfig) -> Result<StrayStream,Box<dyn std::error::Error>> {
    log::info!("Capturing a stray stream at {}.",cfg.target);
    #[cfg(unix)]
    if let Some(ref path) = cfg.unix {
      let path = path.strip_prefix("unix://").unwrap_or(&path);
      let unix_stream: UnixStream = StraySocket::connect_unix(path)?.try_into()?;
      return if cfg.secure {
        let mut builder = SslConnector::builder(SslMethod::tls())?;
        builder.set_verify(SslVerifyMode::NONE);
        let connector = builder.build();
        let mut ssl_config = connector.configure()?;
        ssl_config.set_verify_hostname(false);
        Ok(StrayStream::UnixTls(ssl_config.connect("localhost", unix_stream)?))
      } else {
        Ok(StrayStream::Unix(unix_stream))
      };
    }
    let tcp_stream: TcpStream = StraySocket::connect_tcp(cfg)?.try_into()?;
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

  pub fn listen(cfg: &StrayConfig) -> Result<StrayStream, Box<dyn std::error::Error>> {
    #[cfg(unix)]
    if let Some(ref unix_path) = cfg.unix {
      let path = unix_path.strip_prefix("unix://").unwrap_or(unix_path);
      let unix_stream: UnixStream = StraySocket::listen_unix(path, cfg.verbose)?.try_into()?;
      return Ok(StrayStream::Unix(unix_stream));
    }
    let tcp_stream: TcpStream = StraySocket::listen_tcp(cfg)?.try_into()?;
    Ok(StrayStream::Tcp(tcp_stream))
  }

  pub(crate) fn run(cfg: &StrayConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = if cfg.listen {
      Self::listen(cfg)?
    } else {
      Self::connect(cfg)?
    };
    log::info!("Connected: {}",stream);
    if cfg.verbose {
      eprintln!("Connected: {}",stream);
      stream.print_cert();
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
      StrayStream::Tls(stream) => stream.ssl().peer_certificate(),
      #[cfg(unix)]
      StrayStream::UnixTls(s) => s.ssl().peer_certificate(),
      _ => None,
    }
  }

  pub fn verify_cert(&self) -> Option<X509VerifyResult> {
    match self {
      StrayStream::Tls(s) => Some(s.ssl().verify_result()),
      #[cfg(unix)]
      StrayStream::UnixTls(s) => Some(s.ssl().verify_result()),
      _ => None,
    }
  }

  pub fn print_cert(&self) {
    let Some(cert) = self.get_cert() else { return };
    eprintln!("* Server certificate:");
    let format_name = |name: &openssl::x509::X509NameRef| -> String {
      name.entries().filter_map(|e| {
        let key   = e.object().nid().short_name().ok()?;
        let value = e.data().to_string().ok()?;
        Some(format!("{}={}",key,value))
      }).collect::<Vec<_>>().join(", ")
    };
    eprintln!("*  subject: {}", format_name(cert.subject_name()));
    eprintln!("*  start date: {}", cert.not_before());
    eprintln!("*  expire date: {}", cert.not_after());
    if let Some(sans) = cert.subject_alt_names() {
      let names: Vec<String> = sans.iter().filter_map(|g| { g.dnsname().map(|dns| format!("DNS:{}",dns)).or_else(|| g.ipaddress().map(|ip| format!("IP:{ip:?}"))) }).collect();
      if !names.is_empty() { eprintln!("*  subjectAltName: {}", names.join(", ")); }
    }
    eprintln!("*  issuer: {}", format_name(cert.issuer_name()));
    if let Some(verify) = self.verify_cert() { eprintln!("*  SSL certificate verify result: {verify}"); }
  }

  pub fn tcp_ref(&self) -> Option<&TcpStream> {
    match self {
      StrayStream::Tcp(stream) => Some(stream),
      StrayStream::Tls(stream) => Some(stream.get_ref()),
      #[cfg(unix)]
      StrayStream::Unix(_) | StrayStream::UnixTls(_) => None,
    }
  }

  pub fn peer_addr(&self) -> io::Result<SocketAddr> {
    match self {
      StrayStream::Tcp(stream) => stream.peer_addr(),
      StrayStream::Tls(stream) => stream.get_ref().peer_addr(),
      #[cfg(unix)]
      StrayStream::Unix(_) | StrayStream::UnixTls(_) => {
        Err(io::Error::new(io::ErrorKind::AddrNotAvailable, "Unix domain socket has no IP SocketAddr"))
      }
    }
  }

  pub fn local_addr(&self) -> io::Result<SocketAddr> {
    match self {
      StrayStream::Tcp(stream) => stream.local_addr(),
      StrayStream::Tls(stream) => stream.get_ref().local_addr(),
      #[cfg(unix)]
      StrayStream::Unix(_) | StrayStream::UnixTls(_) => {
        Err(io::Error::new(io::ErrorKind::AddrNotAvailable, "Unix domain socket has no IP SocketAddr"))
      }
    }
  }

  pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
    match self {
      StrayStream::Tcp(s) => s.set_nodelay(nodelay),
      StrayStream::Tls(s) => s.get_ref().set_nodelay(nodelay),
      // TCP_NODELAY does not apply to Unix domain sockets
      #[cfg(unix)]
      StrayStream::Unix(_) | StrayStream::UnixTls(_) => Ok(()),
    }
  }

  pub fn is_tls(&self) -> bool {
    match self {
      StrayStream::Tls(_) => true,
      #[cfg(unix)]
      StrayStream::UnixTls(_) => true,
      _ => false,
    }
  }

  fn fmt_tls(f: &mut fmt::Formatter<'_>, prefix: &str, ssl: &openssl::ssl::SslRef) -> fmt::Result {
    let version = ssl.version_str();
    let cipher = ssl.current_cipher().map(|c| c.name()).unwrap_or("unknown");
    write!(f, "{prefix} [{version}, {cipher}")?;
    if let Some(sni) = ssl.servername(NameType::HOST_NAME) {
      write!(f, ", sni={sni}")?;
    }
    if let Some(alpn) = ssl.selected_alpn_protocol() {
      write!(f, ", alpn={}", String::from_utf8_lossy(alpn))?;
    }
    write!(f, "]")
  }
}
