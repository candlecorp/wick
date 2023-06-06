use std::path::PathBuf;

use url::Url;
use wick_asset_reference::AssetReference;

use crate::error::ManifestError;

#[derive(Debug, Clone, Builder, derive_asset_container::AssetManager, property::Property)]
#[asset(asset(AssetReference))]
#[property(get(public), set(private), mut(disable))]
/// A definition of a Wick Collection with its namespace, how to retrieve or access it and its configuration.
#[must_use]
pub struct ResourceBinding {
  #[asset(skip)]
  /// The id to bind the resource to.
  pub(crate) id: String,
  /// The bound resource.
  pub(crate) kind: ResourceDefinition,
}

impl ResourceBinding {
  /// Create a new [ResourceBinding] with specified name and [ResourceDefinition].
  pub fn new(name: impl AsRef<str>, kind: ResourceDefinition) -> Self {
    Self {
      id: name.as_ref().to_owned(),
      kind,
    }
  }
}

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]
/// Normalized representation of a resource definition.
pub enum ResourceDefinition {
  /// A TCP port.
  #[asset(skip)]
  TcpPort(TcpPort),
  /// A UDP port.
  #[asset(skip)]
  UdpPort(UdpPort),
  /// A URL resource.
  #[asset(skip)]
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
      _ => panic!("Cannot convert non-URL resource to URL"),
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

#[derive(Debug, Clone, PartialEq, Builder, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference), lazy)]
/// A filesystem or network volume.
#[must_use]
pub struct Volume {
  pub(crate) path: AssetReference,
}

impl Volume {
  /// Create a new Volume.
  pub fn new(path: impl AsRef<str>) -> Self {
    Self {
      path: AssetReference::new(path.as_ref()),
    }
  }

  pub fn path(&self) -> Result<PathBuf, ManifestError> {
    Ok(self.path.path()?)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, property::Property)]
/// A URL resource.
#[must_use]
#[property(get(public), set(private), mut(disable))]
pub struct UrlResource {
  /// The URL
  pub(crate) url: Url,
}

impl UrlResource {
  /// Create a new URL Resource.
  pub fn new(url: Url) -> Self {
    Self { url }
  }

  /// Get the URL.
  #[must_use]
  pub fn into_inner(self) -> Url {
    self.url
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

impl std::ops::Deref for UrlResource {
  type Target = Url;

  fn deref(&self) -> &Self::Target {
    &self.url
  }
}

impl std::fmt::Display for UrlResource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.url)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, property::Property)]
/// Normalized representation of a TCP port configuration.
#[property(get(public), set(private), mut(disable))]
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

  /// Get the address and port as a string.
  #[must_use]
  pub fn address(&self) -> String {
    format!("{}:{}", self.host, self.port)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, property::Property)]
/// Normalized representation of a UDP port configuration.
#[property(get(public), set(private), mut(disable))]
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

  /// Get the address and port as a string.
  #[must_use]
  pub fn address(&self) -> String {
    format!("{}:{}", self.host, self.port)
  }
}
