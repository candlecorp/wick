#![allow(missing_docs)] // delete when we move away from the `property` crate.
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use asset_container::AssetManager;
use tracing::trace;
use wick_asset_reference::AssetReference;

use crate::audit::Audit;
use crate::config::template_config::Renderable;
use crate::{config, Result};
mod resources;
pub use resources::*;

#[derive(
  Debug, Clone, derive_builder::Builder, derive_asset_container::AssetManager, property::Property, serde::Serialize,
)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[asset(asset(AssetReference))]
#[must_use]
/// A Wick lockdown configuration.
///
/// A lockdown configuration defines what a wick application can or can't do.
pub struct LockdownConfiguration {
  /// The source (i.e. url or file on disk) of the configuration.
  #[asset(skip)]
  #[property(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) source: Option<PathBuf>,

  /// Any metadata associated with the configuration.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) metadata: Option<config::Metadata>,

  /// Resources and how to restrict them.
  #[asset(skip)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) resources: Vec<ResourceRestriction>,

  /// The environment this configuration has access to.
  #[asset(skip)]
  #[builder(default)]
  #[property(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) env: Option<HashMap<String, String>>,
}

impl LockdownConfiguration {
  /// Return the version of the application.
  #[must_use]
  pub fn version(&self) -> Option<&str> {
    self.metadata.as_ref().map(|m| m.version.as_str())
  }

  /// Initialize the configuration.
  pub(crate) fn initialize(&mut self) -> Result<&Self> {
    // This pre-renders the component config's resources without access to the environment.
    trace!(
      resource_restrictions = ?self.resources,
      "initializing lockdown configuration"
    );
    self
      .resources
      .render_config(self.source.as_deref(), None, self.env.as_ref())?;

    Ok(self)
  }

  /// Set the source location of the configuration.
  pub fn set_source(&mut self, source: &Path) {
    let source = source.to_path_buf();
    self.source = Some(source);
  }

  pub(super) fn update_baseurls(&self) {
    #[allow(clippy::expect_used)]
    let mut source = self.source.clone().expect("No source set for this configuration");
    // Source is (should be) a file, so pop the filename before setting the baseurl.
    if !source.is_dir() {
      source.pop();
    }
    self.set_baseurl(&source);
  }

  /// Return the environment variables for this configuration.
  #[must_use]
  pub const fn env(&self) -> Option<&HashMap<String, String>> {
    None
  }

  /// Validate this configuration is good.
  #[allow(clippy::missing_const_for_fn)]
  pub fn validate(&self) -> Result<()> {
    /* placeholder */
    Ok(())
  }
}

impl Renderable for LockdownConfiguration {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&wick_packet::RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<()> {
    self.resources.render_config(source, root_config, env)?;
    Ok(())
  }
}

impl From<Vec<Audit>> for LockdownConfiguration {
  fn from(value: Vec<Audit>) -> Self {
    let mut url_restrictions: Vec<UrlRestriction> = Vec::new();
    let mut volume_restrictions: Vec<VolumeRestriction> = Vec::new();
    let mut tcpport_restrictions: Vec<PortRestriction> = Vec::new();
    let mut udpport_restrictions: Vec<PortRestriction> = Vec::new();
    let mut restrictions: Vec<ResourceRestriction> = Vec::new();
    let mut reverse_map: HashMap<&crate::audit::AuditedResource, HashSet<&String>> = HashMap::new();

    for audit in &value {
      for resource in &audit.resources {
        let entry = reverse_map.entry(&resource.resource).or_insert_with(HashSet::new);
        entry.insert(&audit.name);
      }
    }

    for (resource, components) in reverse_map {
      let components: Vec<_> = components.iter().map(|s| (*s).clone()).collect();
      match resource {
        crate::audit::AuditedResource::TcpPort(v) => tcpport_restrictions.push(PortRestriction::new_from_template(
          components,
          &v.address,
          v.port.to_string(),
        )),
        crate::audit::AuditedResource::UdpPort(v) => udpport_restrictions.push(PortRestriction::new_from_template(
          components,
          &v.address,
          v.port.to_string(),
        )),
        crate::audit::AuditedResource::Url(v) => {
          url_restrictions.push(UrlRestriction::new_from_template(components, v.url.as_str()));
        }
        crate::audit::AuditedResource::Volume(v) => volume_restrictions.push(VolumeRestriction::new_from_template(
          components,
          v.path.to_string_lossy(),
        )),
      }
    }

    restrictions.extend(tcpport_restrictions.into_iter().map(ResourceRestriction::TcpPort));
    restrictions.extend(udpport_restrictions.into_iter().map(ResourceRestriction::UdpPort));
    restrictions.extend(url_restrictions.into_iter().map(ResourceRestriction::Url));
    restrictions.extend(volume_restrictions.into_iter().map(ResourceRestriction::Volume));

    LockdownConfiguration {
      source: None,
      metadata: None,
      resources: restrictions,
      env: None,
    }
  }
}
