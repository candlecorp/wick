use std::collections::HashMap;
use std::path::PathBuf;

use asset_container::{Asset, AssetFlags};
use url::Url;
use wick_asset_reference::AssetReference;
use wick_packet::RuntimeConfig;

use crate::config::TemplateConfig;
use crate::error::ManifestError;

#[derive(Debug, Clone, Builder, derive_asset_container::AssetManager, property::Property)]
#[asset(asset(AssetReference))]
#[builder(derive(Debug), setter(into))]
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

impl ResourceDefinition {
  /// Render the resource configuration
  pub(crate) fn render_config(
    &mut self,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    match self {
      ResourceDefinition::TcpPort(v) => v.render_config(root_config, env),
      ResourceDefinition::UdpPort(v) => v.render_config(root_config, env),
      ResourceDefinition::Url(v) => v.render_config(root_config, env),
      ResourceDefinition::Volume(v) => v.render_config(root_config, env),
    }
  }
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

#[derive(Debug, Clone, PartialEq, Builder)]
/// A filesystem or network volume.
#[must_use]
pub struct Volume {
  pub(crate) path: TemplateConfig<AssetReference>,
}

impl Volume {
  /// Create a new Volume.
  pub fn new(path: impl AsRef<str>) -> Self {
    Self {
      path: TemplateConfig::new_value(AssetReference::new(path.as_ref())),
    }
  }

  pub fn path(&self) -> Result<PathBuf, ManifestError> {
    if let Some(path) = &self.path.value {
      Ok(path.path()?)
    } else {
      Err(ManifestError::UnrenderedConfiguration(format!(
        "{:?}",
        self.path.template
      )))
    }
  }

  /// Render the resource configuration
  pub(crate) fn render_config(
    &mut self,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.path.set_value(self.path.render(root_config, env)?);
    Ok(())
  }
}

impl asset_container::AssetManager for Volume {
  type Asset = AssetReference;

  fn assets(&self) -> asset_container::Assets<Self::Asset> {
    asset_container::Assets::new(
      self
        .path
        .value
        .as_ref()
        .map_or_else(Vec::new, |v| vec![std::borrow::Cow::Borrowed(v)]),
      self.get_asset_flags(),
    )
  }

  fn set_baseurl(&self, baseurl: &std::path::Path) {
    if let Some(path) = &self.path.value {
      path.update_baseurl(baseurl);
    }
  }

  fn get_asset_flags(&self) -> u32 {
    AssetFlags::Lazy.bits()
  }
}

#[derive(Debug, Clone, PartialEq, property::Property)]
/// A URL resource.
#[must_use]
#[property(get(public), set(private), mut(disable))]
pub struct UrlResource {
  /// The URL
  pub(crate) url: TemplateConfig<Url>,
}

impl UrlResource {
  /// Create a new URL Resource.
  pub fn new(url: Url) -> Self {
    Self {
      url: TemplateConfig::new_value(url),
    }
  }

  /// Get the URL.
  #[must_use]
  pub fn into_inner(self) -> TemplateConfig<Url> {
    self.url
  }

  /// Render the resource configuration
  pub(crate) fn render_config(
    &mut self,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.url.set_value(self.url.render(root_config, env)?);
    Ok(())
  }
}

impl std::fmt::Display for UrlResource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.url)
  }
}

#[derive(Debug, Clone, PartialEq, property::Property)]
/// Normalized representation of a TCP port configuration.
#[property(get(public), set(private), mut(disable))]
pub struct TcpPort {
  /// The port number.
  pub(crate) port: TemplateConfig<u16>,
  /// The address to bind to.
  pub(crate) host: TemplateConfig<String>,
}

impl TcpPort {
  /// Create a new TCP port configuration.
  pub fn new(host: impl AsRef<str>, port: u16) -> Self {
    Self {
      port: TemplateConfig::new_value(port),
      host: TemplateConfig::new_value(host.as_ref().to_owned()),
    }
  }

  /// Get the address and port as a string.
  #[must_use]
  pub fn address(&self) -> String {
    format!("{}:{}", self.host, self.port)
  }

  /// Render the resource configuration
  pub(crate) fn render_config(
    &mut self,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.port.set_value(self.port.render(root_config, env)?);
    self.host.set_value(self.host.render(root_config, env)?);
    Ok(())
  }
}

#[derive(Debug, Clone, PartialEq, property::Property)]
/// Normalized representation of a UDP port configuration.
#[property(get(public), set(private), mut(disable))]
pub struct UdpPort {
  /// The port number.
  pub(crate) port: TemplateConfig<u16>,
  /// The address to bind to.
  pub(crate) host: TemplateConfig<String>,
}

impl UdpPort {
  /// Create a new UDP port configuration.
  pub fn new(host: impl AsRef<str>, port: u16) -> Self {
    Self {
      port: TemplateConfig::new_value(port),
      host: TemplateConfig::new_value(host.as_ref().to_owned()),
    }
  }

  /// Get the address and port as a string.
  #[must_use]
  pub fn address(&self) -> String {
    format!("{}:{}", self.host, self.port)
  }

  /// Render the resource configuration
  pub(crate) fn render_config(
    &mut self,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.port.set_value(self.port.render(root_config, env)?);
    self.host.set_value(self.host.render(root_config, env)?);
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;

  #[test]
  fn try_into_urlresource() -> Result<()> {
    let urlstr = "postgres://postgres:password!12345@nas.glhf.lan:55432/wick_test".to_owned();
    let url_resource: UrlResource = urlstr.clone().try_into()?;
    let url: Url = urlstr.parse()?;

    assert_eq!(url_resource.url.value_unchecked().as_str(), &urlstr);
    assert_eq!(url_resource.url.value_unchecked(), &url);

    Ok(())
  }
}
