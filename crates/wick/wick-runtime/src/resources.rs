use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use wick_config::config::{ResourceDefinition, TcpPort, UdpPort, UrlResource};

#[derive(thiserror::Error, Debug)]
pub enum ResourceError {
  #[error("Invalid IP address '{0}': {1}")]
  InvalidIpAddress(String, String),
}

#[derive(Debug, Clone, PartialEq)]
#[allow(missing_copy_implementations)]
pub enum Resource {
  TcpPort(SocketAddr),
  UdpPort(SocketAddr),
  Url(UrlResource),
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[must_use]
pub enum ResourceKind {
  TcpPort,
  UdpPort,
  Url,
}

impl std::fmt::Display for ResourceKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::TcpPort => write!(f, "TcpPort"),
      Self::UdpPort => write!(f, "UdpPort"),
      Self::Url => write!(f, "Url"),
    }
  }
}

impl Resource {
  pub fn new(config: ResourceDefinition) -> Result<Self, ResourceError> {
    match config {
      ResourceDefinition::TcpPort(config) => Self::new_tcp_port(&config),
      ResourceDefinition::UdpPort(config) => Self::new_udp_port(&config),
      ResourceDefinition::Url(config) => Self::new_url(&config),
    }
  }
  pub fn new_tcp_port(config: &TcpPort) -> Result<Self, ResourceError> {
    Ok(Self::TcpPort(SocketAddr::new(
      IpAddr::from_str(config.host())
        .map_err(|e| ResourceError::InvalidIpAddress(config.host().to_owned(), e.to_string()))?,
      config.port(),
    )))
  }

  pub fn new_udp_port(config: &UdpPort) -> Result<Self, ResourceError> {
    Ok(Self::UdpPort(SocketAddr::new(
      IpAddr::from_str(config.host())
        .map_err(|e| ResourceError::InvalidIpAddress(config.host().to_owned(), e.to_string()))?,
      config.port(),
    )))
  }

  pub fn new_url(config: &UrlResource) -> Result<Self, ResourceError> {
    Ok(Self::Url(config.clone()))
  }

  pub fn kind(&self) -> ResourceKind {
    match self {
      Self::TcpPort(_) => ResourceKind::TcpPort,
      Self::UdpPort(_) => ResourceKind::UdpPort,
      Self::Url(_) => ResourceKind::Url,
    }
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;

  #[test]
  fn test_basic() -> Result<()> {
    let resource = Resource::new_tcp_port(&TcpPort::new("0.0.0.0", 8888))?;
    assert_eq!(
      resource,
      Resource::TcpPort(SocketAddr::new(IpAddr::from_str("0.0.0.0")?, 8888))
    );

    Ok(())
  }
}
