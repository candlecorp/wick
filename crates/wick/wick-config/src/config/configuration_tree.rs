#![allow(missing_docs)]


use wick_asset_reference::FetchOptions;
use wick_packet::RuntimeConfig;
use wick_packet::RuntimeConfig;

use super::{ImportDefinition, UninitializedConfiguration};
use crate::config::{Binding, ComponentDefinition};
use crate::error::ManifestError;
use crate::{Imports, WickConfiguration};
use crate::{Imports, WickConfiguration};

#[derive(Debug)]

pub enum ConfigOrDefinition<T> {
  Config(ConfigurationTreeNode<T>),
pub enum ConfigOrDefinition<T> {
  Config(ConfigurationTreeNode<T>),
  Definition { id: String, element: ComponentDefinition },
}

impl<T> ConfigOrDefinition<T> {
impl<T> ConfigOrDefinition<T> {
  /// Returns the held configuration if it is a [ConfigurationTreeNode].
  #[must_use]
  #[allow(clippy::missing_const_for_fn)]
  pub fn as_config(self) -> Option<T> {
  pub fn as_config(self) -> Option<T> {
    match self {
      ConfigOrDefinition::Config(c) => Some(c.element),
      ConfigOrDefinition::Definition { .. } => None,
    }
  }
  /// Returns the held configuration if it is a [ComponentDefinition].
  #[must_use]
  #[allow(clippy::missing_const_for_fn)]
  pub fn as_component_definition(self) -> Option<(String, ComponentDefinition)> {
    match self {
      ConfigOrDefinition::Config(_) => None,
      ConfigOrDefinition::Definition { id, element } => Some((id, element)),
    }
  }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct ConfigurationTreeNode<T> {
pub struct ConfigurationTreeNode<T> {
  pub name: String,
  pub element: T,
  pub children: Vec<ConfigOrDefinition<T>>,
  pub element: T,
  pub children: Vec<ConfigOrDefinition<T>>,
}

impl<T> ConfigurationTreeNode<T>
where
  T: Imports + Send + Sync,
{
impl<T> ConfigurationTreeNode<T>
where
  T: Imports + Send + Sync,
{
  #[must_use]
  pub const fn new(name: String, element: T, children: Vec<ConfigOrDefinition<T>>) -> Self {
  pub const fn new(name: String, element: T, children: Vec<ConfigOrDefinition<T>>) -> Self {
    Self {
      name,
      element,
      children,
      children,
    }
  }

  /// Flattens a configuration tree into a list of its namespaces elements.
  #[must_use]
  pub fn flatten(self) -> Vec<ConfigOrDefinition<T>> {
  pub fn flatten(self) -> Vec<ConfigOrDefinition<T>> {
    let id = self.name.clone();
    flatten(self, &id)
  }
}
}

#[async_recursion::async_recursion]
pub async fn fetch_children<T, H, U>(
  config: &T,
  options: FetchOptions,
  processor: &H,
) -> Result<Vec<ConfigOrDefinition<U>>, ManifestError>
where
  T: Imports + Send + Sync,
  H: Fn(Option<RuntimeConfig>, UninitializedConfiguration) -> Result<U, ManifestError> + Send + Sync,
  U: Imports + Send + Sync,
{
  let imports = config.imports().to_vec();
#[async_recursion::async_recursion]
pub async fn fetch_children<T, H, U>(
  config: &T,
  options: FetchOptions,
  processor: &H,
) -> Result<Vec<ConfigOrDefinition<U>>, ManifestError>
where
  T: Imports + Send + Sync,
  H: Fn(Option<RuntimeConfig>, UninitializedConfiguration) -> Result<U, ManifestError> + Send + Sync,
  U: Imports + Send + Sync,
{
  let imports = config.imports().to_vec();

  let mut children = fetch_imports(imports, options.clone(), processor).await?;
  for child in &mut children {
    match child {
      ConfigOrDefinition::Config(ref mut c) => {
        let children = fetch_children(&c.element, options.clone(), processor).await?;
        c.children = children;
      }
      ConfigOrDefinition::Definition { .. } => {}
    }
  }
  Ok(children)
  let mut children = fetch_imports(imports, options.clone(), processor).await?;
  for child in &mut children {
    match child {
      ConfigOrDefinition::Config(ref mut c) => {
        let children = fetch_children(&c.element, options.clone(), processor).await?;
        c.children = children;
      }
      ConfigOrDefinition::Definition { .. } => {}
    }
  }
  Ok(children)
}

#[must_use]
pub fn flatten<T>(node: ConfigurationTreeNode<T>, prefix: &str) -> Vec<ConfigOrDefinition<T>> {
pub fn flatten<T>(node: ConfigurationTreeNode<T>, prefix: &str) -> Vec<ConfigOrDefinition<T>> {
  let mut flattened = Vec::new();
  let children = node.children;
  let new_node = ConfigurationTreeNode {
    name: prefix.to_owned(),
    element: node.element,
    children: Vec::new(),
  };
  flattened.push(ConfigOrDefinition::Config(new_node));
  for child in children {
    match child {
      ConfigOrDefinition::Config(c) => {
        let id = format!("{}::{}", prefix, c.name);
        let new_node = ConfigurationTreeNode {
          name: id.clone(),
          element: c.element,
          children: c.children,
        };

        flattened.extend(flatten(new_node, &id));
      }
      ConfigOrDefinition::Definition { .. } => flattened.push(child),
    }
  }
  flattened
}

async fn fetch_imports<H, T>(
  imports: Vec<Binding<ImportDefinition>>,
  options: FetchOptions,
  processor: &H,
) -> Result<Vec<ConfigOrDefinition<T>>, ManifestError>
where
  T: Imports + Send + Sync,
  H: Fn(Option<RuntimeConfig>, UninitializedConfiguration) -> Result<T, ManifestError> + Send + Sync,
{
  let mut children: Vec<ConfigOrDefinition<T>> = Vec::new();
  processor: &H,
) -> Result<Vec<ConfigOrDefinition<T>>, ManifestError>
where
  T: Imports + Send + Sync,
  H: Fn(Option<RuntimeConfig>, UninitializedConfiguration) -> Result<T, ManifestError> + Send + Sync,
{
  let mut children: Vec<ConfigOrDefinition<T>> = Vec::new();
  for import in imports {
    let id = import.id().to_owned();

    match import.kind {
      super::ImportDefinition::Component(c) => match c {
        super::ComponentDefinition::Manifest(c) => {
          let config = WickConfiguration::fetch(c.reference.clone(), options.clone()).await?;
          let config = processor(c.config().and_then(|c| c.value.clone()), config)?;
          let config = WickConfiguration::fetch(c.reference.clone(), options.clone()).await?;
          let config = processor(c.config().and_then(|c| c.value.clone()), config)?;

          children.push(ConfigOrDefinition::Config(ConfigurationTreeNode::new(
            id,
            config,
            Vec::new(),
          )));
          children.push(ConfigOrDefinition::Config(ConfigurationTreeNode::new(
            id,
            config,
            Vec::new(),
          )));
        }
        component => {
          children.push(ConfigOrDefinition::Definition {
            id,
            element: component.clone(),
          });
        }
      },
      super::ImportDefinition::Types(_) => {}
    }
  }
  Ok(children)
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;
  use crate::config::{components, AppConfigurationBuilder, Binding, ComponentDefinition, ImportDefinition};
  use crate::config::{components, AppConfigurationBuilder, Binding, ComponentDefinition, ImportDefinition};

  #[test_logger::test(tokio::test)]
  async fn test_tree_walker() -> Result<()> {
    let mut config = AppConfigurationBuilder::default();

    config
      .name("app")
      .options(FetchOptions::default())
      .import(vec![Binding::new(
      .import(vec![Binding::new(
        "SUB_COMPONENT",
        ImportDefinition::Component(ComponentDefinition::Manifest(
          components::ManifestComponentBuilder::default()
            .reference("tests/manifests/v1/component-resources.yaml")
            .build()?,
        )),
      )]);
    let config = config.build()?;
    let config = UninitializedConfiguration::new(WickConfiguration::App(config));
    let children = fetch_children(&config, Default::default(), &|_, c| Ok(c)).await?;
    assert_eq!(children.len(), 1);
    let config = UninitializedConfiguration::new(WickConfiguration::App(config));
    let children = fetch_children(&config, Default::default(), &|_, c| Ok(c)).await?;
    assert_eq!(children.len(), 1);
    Ok(())
  }
}
