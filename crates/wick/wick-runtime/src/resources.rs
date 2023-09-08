use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

use url::Url;
use wick_config::config::{ResourceDefinition, TcpPort, UdpPort, UrlResource, Volume};

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum ResourceError {
  #[error("Invalid IP address '{0}': {1}")]
  InvalidIpAddress(String, String),
  #[error("Invalid path: {0}")]
  InvalidPath(String),
}

#[derive(Debug, Clone, PartialEq)]
#[allow(missing_copy_implementations)]
#[allow(clippy::exhaustive_enums)]
pub enum Resource {
  TcpPort(SocketAddr),
  UdpPort(SocketAddr),
  Url(Url),
  Volume(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[must_use]
#[allow(clippy::exhaustive_enums)]
pub enum ResourceKind {
  TcpPort,
  UdpPort,
  Url,
  Volume,
}

impl std::fmt::Display for ResourceKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::TcpPort => write!(f, "TcpPort"),
      Self::UdpPort => write!(f, "UdpPort"),
      Self::Url => write!(f, "Url"),
      Self::Volume => write!(f, "Volume"),
    }
  }
}

impl Resource {
  pub fn new(config: ResourceDefinition) -> Result<Self, ResourceError> {
    match config {
      ResourceDefinition::TcpPort(config) => Self::new_tcp_port(&config),
      ResourceDefinition::UdpPort(config) => Self::new_udp_port(&config),
      ResourceDefinition::Url(config) => Self::new_url(&config),
      ResourceDefinition::Volume(config) => Self::new_volume(&config),
    }
  }

  pub fn new_tcp_port(config: &TcpPort) -> Result<Self, ResourceError> {
    let host = config.host().value_unchecked();
    let port = config.port().value_unchecked();
    Ok(Self::TcpPort(SocketAddr::new(
      IpAddr::from_str(host).map_err(|e| ResourceError::InvalidIpAddress(host.clone(), e.to_string()))?,
      *port,
    )))
  }

  pub fn new_udp_port(config: &UdpPort) -> Result<Self, ResourceError> {
    let host = config.host().value_unchecked();
    let port = config.port().value_unchecked();
    Ok(Self::UdpPort(SocketAddr::new(
      IpAddr::from_str(host).map_err(|e| ResourceError::InvalidIpAddress(host.clone(), e.to_string()))?,
      *port,
    )))
  }

  pub fn new_url(config: &UrlResource) -> Result<Self, ResourceError> {
    Ok(Self::Url(config.url().value_unchecked().clone()))
  }

  pub fn new_volume(config: &Volume) -> Result<Self, ResourceError> {
    Ok(Self::Volume(
      config.path().map_err(|e| ResourceError::InvalidPath(e.to_string()))?,
    ))
  }

  pub const fn kind(&self) -> ResourceKind {
    match self {
      Self::TcpPort(_) => ResourceKind::TcpPort,
      Self::UdpPort(_) => ResourceKind::UdpPort,
      Self::Url(_) => ResourceKind::Url,
      Self::Volume(_) => ResourceKind::Volume,
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
