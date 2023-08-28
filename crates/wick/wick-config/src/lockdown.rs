mod error;
mod port;
mod url;
mod volume;

pub use error::*;

use crate::audit::{Audit, AuditedResource, AuditedResourceBinding};
use crate::config::{ConfigOrDefinition, LockdownConfiguration, ResourceRestriction};

pub(crate) fn validate_resource(
  component_id: &str,
  resource: &AuditedResourceBinding,
  lockdown: &LockdownConfiguration,
) -> Result<(), LockdownError> {
  let resource_restrictions = lockdown.resources();
  match &resource.resource {
    AuditedResource::TcpPort(v) => self::port::validate(
      component_id,
      &resource.name,
      v,
      resource_restrictions.iter().filter_map(|r| match r {
        ResourceRestriction::TcpPort(p) => Some(p),
        _ => None,
      }),
    ),
    AuditedResource::UdpPort(v) => self::port::validate(
      component_id,
      &resource.name,
      v,
      resource_restrictions.iter().filter_map(|r| match r {
        ResourceRestriction::UdpPort(p) => Some(p),
        _ => None,
      }),
    ),
    AuditedResource::Url(v) => self::url::validate(
      component_id,
      &resource.name,
      v,
      resource_restrictions.iter().filter_map(|r| match r {
        ResourceRestriction::Url(v) => Some(v),
        _ => None,
      }),
    ),
    AuditedResource::Volume(v) => self::volume::validate(
      component_id,
      &resource.name,
      v,
      resource_restrictions.iter().filter_map(|r| match r {
        ResourceRestriction::Volume(v) => Some(v),
        _ => None,
      }),
    ),
  }
}

/// Apply lockdown restrictions to a configuration tree.
pub fn assert_restrictions(
  elements: &[ConfigOrDefinition],
  lockdown: &LockdownConfiguration,
) -> Result<(), LockdownError> {
  let audit = elements.iter().map(Audit::config_or_def).collect::<Vec<_>>();
  for element in audit {
    for resource in element.resources {
      validate_resource(&element.name, &resource, lockdown)?;
    }
  }
  Ok(())
}

/// The [Lockdown] trait defines the interface for applying lockdowns to a configuration.
pub trait Lockdown {
  /// Apply a lockdown configuration to the current configuration.
  fn lockdown(&self, id: Option<&str>, lockdown: &LockdownConfiguration) -> Result<(), LockdownError>;
}

#[cfg(test)]
mod test {
  use std::path::PathBuf;

  use anyhow::Result;
  use normpath::PathExt;

  use super::*;
  use crate::config::{
    components,
    AppConfigurationBuilder,
    ComponentConfiguration,
    ComponentDefinition,
    ConfigurationTreeNode,
    ImportBinding,
    ImportDefinition,
    LockdownConfigurationBuilder,
    ResourceRestriction,
    VolumeRestriction,
  };
  use crate::WickConfiguration;

  fn new_lockdown_config(restrictions: Vec<ResourceRestriction>) -> LockdownConfiguration {
    let mut config = LockdownConfigurationBuilder::default()
      .source(Some("test_component".into()))
      .resources(restrictions)
      .build()
      .unwrap();
    config.initialize().unwrap();
    config
  }

  fn resolve_dir(path: &str) -> String {
    PathBuf::from(path)
      .normalize()
      .unwrap()
      .into_path_buf()
      .to_string_lossy()
      .to_string()
  }

  async fn load_component(path: &str) -> Result<ComponentConfiguration> {
    Ok(
      WickConfiguration::fetch(path, Default::default())
        .await?
        .finish()?
        .try_component_config()?,
    )
  }

  #[test_logger::test(tokio::test)]
  async fn test_tree_walker() -> Result<()> {
    let mut config = AppConfigurationBuilder::default();

    config.name("app").import(vec![ImportBinding::new(
      "SUB_COMPONENT",
      ImportDefinition::Component(ComponentDefinition::Manifest(
        components::ManifestComponentBuilder::default()
          .reference("tests/manifests/v1/component-resources.yaml")
          .build()?,
      )),
    )]);
    let config = config.build()?;
    let mut tree = ConfigurationTreeNode::new("ROOT".to_owned(), WickConfiguration::App(config));
    tree.fetch_children(Default::default()).await?;
    assert_eq!(tree.children.len(), 1);
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_lockdown_fail() -> Result<()> {
    let config = load_component("./tests/manifests/v1/component-resources.yaml").await?;
    let pwd = std::env::current_dir()?;
    let lockdown = new_lockdown_config(vec![ResourceRestriction::Volume(VolumeRestriction::new_from_template(
      vec!["test_component".into()],
      pwd.to_string_lossy(),
    ))]);
    let error = config.lockdown(Some("test_component"), &lockdown).unwrap_err();
    assert_eq!(error.failures().len(), 1);

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_lockdown_pass() -> Result<()> {
    let config = load_component("./tests/manifests/v1/component-resources.yaml").await?;
    let lockdown = new_lockdown_config(vec![ResourceRestriction::Volume(VolumeRestriction::new_from_template(
      vec!["test_component".into()],
      resolve_dir("/etc"),
    ))]);
    let result = config.lockdown(Some("test_component"), &lockdown);
    println!("{:?}", result);
    assert!(result.is_ok());

    Ok(())
  }
}