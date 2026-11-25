use openssl::{
  ssl::{NameType,SslConnector,SslMethod,SslStream,SslVerifyMode},
  x509::{X509,X509VerifyResult},
};
use std::{
  fmt,
  io::{self,Read,Write},
  net::SocketAddr,
  net::TcpStream,
  time::Duration,
};

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
  pub fn connect(target: &str, port: u16, secure: bool) -> Result<StrayStream,Box<dyn std::error::Error>> {
    let tcp_stream = TcpStream::connect((target,port))?;
    if secure {
      let mut builder = SslConnector::builder(SslMethod::tls())?;
      builder.set_verify(SslVerifyMode::NONE);
      let connector = builder.build();
      let mut ssl_config = connector.configure()?;
      ssl_config.set_verify_hostname(false);
      Ok(StrayStream::Tls(ssl_config.connect(target, tcp_stream)?))
    } else {
      Ok(StrayStream::Tcp(tcp_stream))
    }
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
