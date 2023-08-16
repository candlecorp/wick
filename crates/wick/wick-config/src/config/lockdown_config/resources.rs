#![allow(missing_docs)] // delete when we move away from the `property` crate.

use std::collections::HashMap;

use wick_packet::RuntimeConfig;

use crate::config::template_config::Renderable;
use crate::config::TemplateConfig;
use crate::error::ManifestError;

#[derive(Debug, Clone, serde::Serialize)]
#[must_use]
pub enum ResourceRestriction {
  Volume(VolumeRestriction),
  Url(UrlRestriction),
  TcpPort(PortRestriction),
  UdpPort(PortRestriction),
}

impl Renderable for ResourceRestriction {
  fn render_config(
    &mut self,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    match self {
      Self::Volume(restriction) => restriction.render_config(root_config, env),
      Self::Url(restriction) => restriction.render_config(root_config, env),
      Self::TcpPort(restriction) | Self::UdpPort(restriction) => restriction.render_config(root_config, env),
    }
  }
}

#[derive(Debug, Clone, property::Property, serde::Serialize)]
#[property(get(public), set(private), mut(disable))]
/// Settings that define restrictions on what Volumes can be accessed.
pub struct VolumeRestriction {
  /// The components that apply to this restriction.
  pub(crate) components: Vec<String>,
  /// The volumes this restriction allows access to.
  pub(crate) allow: TemplateConfig<String>,
}

impl VolumeRestriction {
  /// Create a new [VolumeRestriction] for the passed components.
  #[must_use]
  pub fn new_from_template(components: Vec<String>, allow: impl Into<String>) -> Self {
    Self {
      components,
      allow: TemplateConfig::new_template(allow.into()),
    }
  }
}

impl Renderable for VolumeRestriction {
  fn render_config(
    &mut self,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.allow.set_value(self.allow.render(root_config, env)?);
    Ok(())
  }
}

#[derive(Debug, Clone, property::Property, serde::Serialize)]
#[property(get(public), set(private), mut(disable))]
/// Settings that define restrictions on what Urls can be accessed.
pub struct UrlRestriction {
  /// The components that apply to this restriction.
  pub(crate) components: Vec<String>,
  /// A regular expression that defines what urls are allowed.
  pub(crate) allow: TemplateConfig<String>,
}

impl UrlRestriction {
  /// Create a new [UrlRestriction] for the passed components.
  #[must_use]
  pub fn new_from_template(components: Vec<String>, allow: impl Into<String>) -> Self {
    Self {
      components,
      allow: TemplateConfig::new_template(allow.into()),
    }
  }
}

impl Renderable for UrlRestriction {
  fn render_config(
    &mut self,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.allow.set_value(self.allow.render(root_config, env)?);
    Ok(())
  }
}

#[derive(Debug, Clone, property::Property, serde::Serialize)]
#[property(get(public), set(private), mut(disable))]
pub struct PortRestriction {
  /// The components that apply to this restriction.
  pub(crate) components: Vec<String>,
  /// The IP address this restriction applies to.
  pub(crate) address: TemplateConfig<String>,
  /// The port this restriction applies to.
  pub(crate) port: TemplateConfig<String>,
}

impl PortRestriction {
  /// Create a new [PortRestriction] for the passed components.
  #[must_use]
  pub fn new_from_template(components: Vec<String>, address: impl Into<String>, port: impl Into<String>) -> Self {
    Self {
      components,
      address: TemplateConfig::new_template(address.into()),
      port: TemplateConfig::new_template(port.into()),
    }
  }
}
impl Renderable for PortRestriction {
  fn render_config(
    &mut self,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.address.set_value(self.address.render(root_config, env)?);
    self.port.set_value(self.port.render(root_config, env)?);
    Ok(())
  }
}
