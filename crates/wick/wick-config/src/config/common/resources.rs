use std::collections::HashMap;
use std::path::{Path, PathBuf};

use asset_container::{Asset, AssetFlags};
use url::Url;
use wick_asset_reference::AssetReference;
use wick_packet::RuntimeConfig;

use super::template_config::Renderable;
use crate::config::TemplateConfig;
use crate::error::ManifestError;

crate::impl_from_for!(ResourceDefinition, TcpPort);
crate::impl_from_for!(ResourceDefinition, UdpPort);
crate::impl_from_for!(ResourceDefinition, Volume);
crate::impl_from_for!(ResourceDefinition, Url, UrlResource);

#[derive(
  Debug, Clone, derive_builder::Builder, derive_asset_container::AssetManager, property::Property, serde::Serialize,
)]
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

impl Renderable for ResourceBinding {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.kind.render_config(source, root_config, env)
  }
}

impl ResourceBinding {
  /// Create a new [ResourceBinding] with specified name and [ResourceDefinition].
  pub fn new(name: impl AsRef<str>, kind: impl Into<ResourceDefinition>) -> Self {
    Self {
      id: name.as_ref().to_owned(),
      kind: kind.into(),
    }
  }
}

/// A resource type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, derive_asset_container::AssetManager, serde::Serialize, PartialEq, Hash, Eq)]
#[asset(asset(AssetReference))]
/// Normalized representation of a resource definition.
#[serde(rename_all = "kebab-case")]
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

impl Renderable for ResourceDefinition {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    match self {
      ResourceDefinition::TcpPort(v) => v.render_config(source, root_config, env),
      ResourceDefinition::UdpPort(v) => v.render_config(source, root_config, env),
      ResourceDefinition::Url(v) => v.render_config(source, root_config, env),
      ResourceDefinition::Volume(v) => v.render_config(source, root_config, env),
    }
  }
}

impl ResourceDefinition {
  #[must_use]
  pub fn kind(&self) -> ResourceKind {
    match self {
      ResourceDefinition::TcpPort(_) => ResourceKind::TcpPort,
      ResourceDefinition::UdpPort(_) => ResourceKind::UdpPort,
      ResourceDefinition::Url(_) => ResourceKind::Url,
      ResourceDefinition::Volume(_) => ResourceKind::Volume,
    }
  }

  pub fn try_tcpport(self) -> Result<TcpPort, ManifestError> {
    self.try_into()
  }

  pub fn try_udpport(self) -> Result<UdpPort, ManifestError> {
    self.try_into()
  }

  pub fn try_url(self) -> Result<UrlResource, ManifestError> {
    self.try_into()
  }

  pub fn try_volume(self) -> Result<Volume, ManifestError> {
    self.try_into()
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

#[derive(Debug, Clone, PartialEq, Hash, Eq, derive_builder::Builder, serde::Serialize)]
/// A filesystem or network volume.
#[must_use]
pub struct Volume {
  path: TemplateConfig<AssetReference>,
}

impl Volume {
  pub fn new(template: String) -> Self {
    let template = TemplateConfig::new_template(template);
    Self { path: template }
  }

  #[cfg(feature = "v1")]
  pub(crate) fn unrender(&self) -> Result<String, ManifestError> {
    self.path.unrender()
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
}

impl Renderable for Volume {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.path.set_value(self.path.render(source, root_config, env)?);
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

  fn set_baseurl(&self, baseurl: &Path) {
    if let Some(path) = &self.path.value {
      path.update_baseurl(baseurl);
      match self.path() {
        Ok(p) => {
          if !p.is_dir() {
            tracing::warn!(%path,"volume path is not a directory");
          }
        }
        Err(e) => {
          tracing::warn!(%path,error=%e,"volume path could not be resolved");
        }
      }
    }
  }

  fn get_asset_flags(&self) -> u32 {
    AssetFlags::Lazy.bits()
  }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, property::Property, serde::Serialize)]
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
}

impl std::fmt::Display for UrlResource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.url)
  }
}

impl Renderable for UrlResource {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.url.set_value(self.url.render(source, root_config, env)?);
    Ok(())
  }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, property::Property, serde::Serialize)]
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
}

impl Renderable for TcpPort {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.port.set_value(self.port.render(source, root_config, env)?);
    self.host.set_value(self.host.render(source, root_config, env)?);
    Ok(())
  }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, property::Property, serde::Serialize)]
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
}

impl Renderable for UdpPort {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.port.set_value(self.port.render(source, root_config, env)?);
    self.host.set_value(self.host.render(source, root_config, env)?);
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
