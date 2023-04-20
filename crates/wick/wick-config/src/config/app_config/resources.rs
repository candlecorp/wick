use url::Url;

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
  /// A URL resource.
  Url(UrlResource),
  /// A filesystem or network volume.
  Volume(Volume),
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

impl From<ResourceDefinition> for UrlResource {
  fn from(value: ResourceDefinition) -> Self {
    match value {
      ResourceDefinition::Url(v) => v,
      _ => panic!("Cannot convert non-authority resource to authority"),
    }
  }
}

impl TryFrom<String> for UrlResource {
  type Error = crate::Error;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    url::Url::parse(&value)
      .map_err(|_| Self::Error::InvalidUrl(value.clone()))
      .map(Self::new)
  }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A filesystem or network volume.
#[must_use]
pub struct Volume {
  pub(crate) path: String,
}

impl Volume {
  /// Create a new Volume.
  pub fn new(path: impl AsRef<str>) -> Self {
    Self {
      path: path.as_ref().to_owned(),
    }
  }

  /// Get the path.
  #[must_use]
  pub fn path(&self) -> &str {
    &self.path
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A URL resource.
#[must_use]
pub struct UrlResource {
  /// The URL
  pub(crate) url: Url,
}

impl UrlResource {
  /// Create a new URL Resource.
  pub fn new(url: Url) -> Self {
    Self { url }
  }

  /// Get the scheme
  #[must_use]
  pub fn scheme(&self) -> &str {
    self.url.scheme()
  }

  /// Get the port number.
  #[must_use]
  pub fn port(&self) -> Option<u16> {
    self.url.port()
  }

  /// Get the host address.
  #[must_use]
  pub fn host(&self) -> &str {
    self.url.host_str().unwrap_or_default()
  }

  /// Get the username.
  #[must_use]
  pub fn username(&self) -> Option<&str> {
    if self.url.username().is_empty() {
      None
    } else {
      Some(self.url.username())
    }
  }

  /// Get the password.
  #[must_use]
  pub fn password(&self) -> Option<&str> {
    self.url.password()
  }

  /// Get the address and port as a string.
  #[must_use]
  pub fn address(&self) -> String {
    self
      .port()
      .map_or_else(|| self.host().to_owned(), |port| format!("{}:{}", self.host(), port))
  }
}

impl std::fmt::Display for UrlResource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.url)
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
