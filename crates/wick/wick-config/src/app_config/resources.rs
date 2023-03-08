use crate::v1;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Normalized representation of a resource definition.
pub enum ResourceDefinition {
  /// A TCP port.
  TcpPort(TcpPort),
  /// A UDP port.
  UdpPort(UdpPort),
}

impl From<v1::ResourceDefinition> for ResourceDefinition {
  fn from(value: v1::ResourceDefinition) -> Self {
    match value {
      v1::ResourceDefinition::TcpPort(v) => Self::TcpPort(v.into()),
      v1::ResourceDefinition::UdpPort(v) => Self::UdpPort(v.into()),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Normalized representation of a TCP port configuration.
pub struct TcpPort {
  /// The port number.
  pub port: u16,
  /// The address to bind to.
  pub address: String,
}

impl From<v1::TcpPort> for TcpPort {
  fn from(value: v1::TcpPort) -> Self {
    Self {
      port: value.port,
      address: value.address,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Normalized representation of a UDP port configuration.
pub struct UdpPort {
  /// The port number.
  pub port: u16,
  /// The address to bind to.
  pub address: String,
}

impl From<v1::UdpPort> for UdpPort {
  fn from(value: v1::UdpPort) -> Self {
    Self {
      port: value.port,
      address: value.address,
    }
  }
}
