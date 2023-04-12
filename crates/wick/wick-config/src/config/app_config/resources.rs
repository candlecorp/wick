#[derive(Debug, Clone, PartialEq)]
/// A definition of a Wick Collection with its namespace, how to retrieve or access it and its configuration.
#[must_use]
pub struct BoundResource {
  /// The id to bind the resource to.
  pub id: String,
  /// The bound resource.
  pub kind: ResourceDefinition,
}

impl BoundResource {
  /// Create a new [CollectionDefinition] with specified name and type.
  pub fn new(name: impl AsRef<str>, kind: ResourceDefinition) -> Self {
    Self {
      id: name.as_ref().to_owned(),
      kind,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Normalized representation of a resource definition.
pub enum ResourceDefinition {
  /// A TCP port.
  TcpPort(TcpPort),
  /// A UDP port.
  UdpPort(UdpPort),
}

impl From<ResourceDefinition> for TcpPort {
  fn from(value: ResourceDefinition) -> Self {
    match value {
      ResourceDefinition::TcpPort(v) => v,
      _ => panic!("Cannot convert non-tcp port to tcp port"),
    }
  }
}

impl From<ResourceDefinition> for UdpPort {
  fn from(value: ResourceDefinition) -> Self {
    match value {
      ResourceDefinition::UdpPort(v) => v,
      _ => panic!("Cannot convert non-udp port to udp port"),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Normalized representation of a TCP port configuration.
pub struct TcpPort {
  /// The port number.
  pub(crate) port: u16,
  /// The address to bind to.
  pub(crate) host: String,
}

impl TcpPort {
  /// Create a new TCP port configuration.
  pub fn new(host: impl AsRef<str>, port: u16) -> Self {
    Self {
      port,
      host: host.as_ref().to_owned(),
    }
  }

  /// Get the port number.
  #[must_use]
  pub fn port(&self) -> u16 {
    self.port
  }

  /// Get the host address.
  #[must_use]
  pub fn host(&self) -> &str {
    &self.host
  }

  /// Get the address and port as a string.
  #[must_use]
  pub fn address(&self) -> String {
    format!("{}:{}", self.host, self.port)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Normalized representation of a UDP port configuration.
pub struct UdpPort {
  /// The port number.
  pub(crate) port: u16,
  /// The address to bind to.
  pub(crate) host: String,
}

impl UdpPort {
  /// Create a new UDP port configuration.
  pub fn new(host: impl AsRef<str>, port: u16) -> Self {
    Self {
      port,
      host: host.as_ref().to_owned(),
    }
  }

  /// Get the port number.
  #[must_use]
  pub fn port(&self) -> u16 {
    self.port
  }

  /// Get the host address.
  #[must_use]
  pub fn host(&self) -> &str {
    &self.host
  }

  /// Get the address and port as a string.
  #[must_use]
  pub fn address(&self) -> String {
    format!("{}:{}", self.host, self.port)
  }
}
