use socket2::{Domain,Protocol,SockAddr,Socket,Type};
use std::{
  fmt,
  io::{self,Read,Write},
  net::{Ipv6Addr,SocketAddr,TcpStream,ToSocketAddrs,UdpSocket},
  path::Path,
};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
#[cfg(unix)]
use std::os::fd::OwnedFd;

use crate::strayconfig::*;

///////////// StraySocket

pub enum StraySocket {
  Tcp(Socket),
  Udp(Socket),
  #[cfg(unix)]
  Unix(Socket),
}

impl StraySocket {
  pub fn connect_tcp(cfg: &StrayConfig) -> Result<Self, Box<dyn std::error::Error>> {
    let mut addrs = (cfg.target.as_str(),cfg.port).to_socket_addrs()?.filter(|addr| match cfg.ip_version {
      4 => addr.is_ipv4(),
      6 => addr.is_ipv6(),
      _ => true,
    });
    if let Some(ref source) = cfg.source {
      let mut local_addr: SocketAddr = source.parse()?;
      if cfg.ip_version == 6 && local_addr.ip().is_unspecified() && local_addr.is_ipv4() {
        local_addr = SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), local_addr.port());
      }
      if (cfg.ip_version == 4 && !local_addr.is_ipv4()) || (cfg.ip_version == 6 && !local_addr.is_ipv6()) {
        return Err(format!("Source address {local_addr} does not match IPv{}", cfg.ip_version).into());
      }
      let remote_addr = addrs.find(|addr| addr.is_ipv4() == local_addr.is_ipv4()).ok_or("No matching IPv4/IPv6 remote address found for source address")?;
      let domain = Domain::for_address(local_addr);
      let socket = Socket::new(domain,Type::STREAM,Some(Protocol::TCP))?;
      socket.set_reuse_address(true)?;
      socket.bind(&local_addr.into())?;
      socket.connect(&remote_addr.into())?;
      Ok(Self::Tcp(socket))
    } else {
      let mut last_err = None;
      for remote_addr in addrs {
        let domain = Domain::for_address(remote_addr);
        let socket = Socket::new(domain,Type::STREAM,Some(Protocol::TCP))?;
        match socket.connect(&remote_addr.into()) {
          Ok(()) => return Ok(Self::Tcp(socket)),
          Err(e) => last_err = Some(e),
        }
      }
      Err(last_err.map(|e| -> Box<dyn std::error::Error> { e.into() }).unwrap_or_else(|| format!("No matching address found for {}", cfg.target).into()))
    }
  }

  pub fn connect_udp(cfg: &StrayConfig) -> Result<Self, Box<dyn std::error::Error>> {
    let mut addrs = (cfg.target.as_str(),cfg.port).to_socket_addrs()?.filter(|addr| match cfg.ip_version {
      4 => addr.is_ipv4(),
      6 => addr.is_ipv6(),
      _ => true,
    });
    let local_addr: SocketAddr = if let Some(ref source) = cfg.source {
      let mut addr: SocketAddr = source.parse()?;
      if cfg.ip_version == 6 && addr.ip().is_unspecified() && addr.is_ipv4() {
        addr = SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), addr.port());
      }
      if (cfg.ip_version == 4 && !addr.is_ipv4()) || (cfg.ip_version == 6 && !addr.is_ipv6()) {
        return Err(format!("Source address {addr} does not match IPv{}", cfg.ip_version).into());
      }
      addr
    } else if cfg.ip_version == 6 {
      SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0)
    } else {
      "0.0.0.0:0".parse()?
    };
    let remote_addr = addrs.find(|addr| addr.is_ipv4() == local_addr.is_ipv4()).ok_or("No matching IPv4/IPv6 remote address found for UDP socket")?;
    let domain = Domain::for_address(remote_addr);
    let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.bind(&local_addr.into())?;
    socket.connect(&remote_addr.into())?;
    Ok(Self::Udp(socket))
  }

  #[cfg(unix)]
  pub fn connect_unix<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
    let addr = SockAddr::unix(path)?;
    let socket = Socket::new(Domain::UNIX, Type::STREAM, None)?;
    socket.connect(&addr)?;
    Ok(Self::Unix(socket))
  }

  pub fn listen_tcp(cfg: &StrayConfig) -> Result<Self, Box<dyn std::error::Error>> {
    let bind_addr: SocketAddr = if let Some(ref source) = cfg.source {
      let mut addr: SocketAddr = source.parse()?;
      if cfg.ip_version == 6 && addr.ip().is_unspecified() && addr.is_ipv4() {
        addr = SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), addr.port());
      }
      if addr.port() == 0 && cfg.port != 0 { addr.set_port(cfg.port); }
      addr
    } else if !cfg.target.is_empty() {
      (cfg.target.as_str(),cfg.port).to_socket_addrs()? .find(|addr| match cfg.ip_version {
        4 => addr.is_ipv4(),
        6 => addr.is_ipv6(),
        _ => true,
      }).ok_or("No matching IPv4/IPv6 bind address found")?
    } else if cfg.ip_version == 6 {
      SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), cfg.port)
    } else {
      SocketAddr::new(std::net::Ipv4Addr::UNSPECIFIED.into(), cfg.port)
    };
    let domain = Domain::for_address(bind_addr);
    let listener = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
    listener.set_reuse_address(true)?;
    listener.bind(&bind_addr.into())?;
    listener.listen(1)?;
    log::info!("Listening on TCP {bind_addr}...");
    if cfg.verbose { eprintln!("Listening on TCP {bind_addr}..."); }
    let (client_socket, _) = listener.accept()?;
    Ok(Self::Tcp(client_socket))
  }

  #[cfg(unix)]
  pub fn listen_unix<P: AsRef<Path>>(path: P, verbose: bool) -> Result<Self, Box<dyn std::error::Error>> {
    let path_ref = path.as_ref();
    if path_ref.exists() { let _ = std::fs::remove_file(path_ref); }
    let addr = SockAddr::unix(path_ref)?;
    let listener = Socket::new(Domain::UNIX, Type::STREAM, None)?;
    listener.bind(&addr)?;
    listener.listen(1)?;
    log::info!("Listening on UNIX {}...", path_ref.display());
    if verbose { eprintln!("Listening on UNIX {}...", path_ref.display()); }
    let (client_socket, _) = listener.accept()?;
    Ok(Self::Unix(client_socket))
  }

  pub fn inner(&self) -> &Socket {
    match self {
      StraySocket::Tcp(s) | StraySocket::Udp(s) => s,
      #[cfg(unix)]
      StraySocket::Unix(s) => s,
    }
  }

  pub fn into_inner(self) -> Socket {
    match self {
      StraySocket::Tcp(s) | StraySocket::Udp(s) => s,
      #[cfg(unix)]
      StraySocket::Unix(s) => s,
    }
  }
}

impl Read for StraySocket {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    match self {
      StraySocket::Tcp(s) | StraySocket::Udp(s) => s.read(buf),
      #[cfg(unix)]
      StraySocket::Unix(s) => s.read(buf),
    }
  }
}

impl Write for StraySocket {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match self {
      StraySocket::Tcp(s) | StraySocket::Udp(s) => s.write(buf),
      #[cfg(unix)]
      StraySocket::Unix(s) => s.write(buf),
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    match self {
      StraySocket::Tcp(s) | StraySocket::Udp(s) => s.flush(),
      #[cfg(unix)]
      StraySocket::Unix(s) => s.flush(),
    }
  }
}

impl fmt::Display for StraySocket {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let kind = match self {
      StraySocket::Tcp(_) => "TCP",
      StraySocket::Udp(_) => "UDP",
      #[cfg(unix)]
      StraySocket::Unix(_) => "UNIX",
    };
    let local = self.inner().local_addr().ok().and_then(|a| a.as_socket().map(|s| s.to_string())).unwrap_or_else(|| "unknown".into());
    let peer = self.inner().peer_addr().ok().and_then(|a| a.as_socket().map(|s| s.to_string())).unwrap_or_else(|| "unknown".into());
    write!(f, "{kind} ({local} -> {peer})")
  }
}

impl TryFrom<StraySocket> for TcpStream {
  type Error = &'static str;
  fn try_from(socket: StraySocket) -> Result<Self, Self::Error> {
    match socket {
      StraySocket::Tcp(s) => Ok(s.into()),
      _ => Err("Expected a TCP StraySocket"),
    }
  }
}

impl TryFrom<StraySocket> for UdpSocket {
  type Error = &'static str;
  fn try_from(socket: StraySocket) -> Result<Self, Self::Error> {
    match socket {
      StraySocket::Udp(s) => Ok(s.into()),
      _ => Err("Expected a UDP StraySocket"),
    }
  }
}

#[cfg(unix)]
impl TryFrom<StraySocket> for UnixStream {
  type Error = &'static str;
  fn try_from(socket: StraySocket) -> Result<Self,Self::Error> {
    match socket {
      StraySocket::Unix(s) => Ok(OwnedFd::from(s).into()),
      _ => Err("Expected a Unix StraySocket"),
    }
  }
}
