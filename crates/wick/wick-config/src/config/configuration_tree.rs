#![allow(missing_docs)]
use wick_asset_reference::FetchOptions;
use wick_packet::{InherentData, RuntimeConfig};

use crate::config::{ComponentDefinition, ImportBinding};
use crate::error::ManifestError;
use crate::WickConfiguration;

#[derive(Debug)]
pub enum ConfigOrDefinition {
  Config(ConfigurationTreeNode),
  Definition { id: String, element: ComponentDefinition },
}

impl ConfigOrDefinition {
  /// Returns the held configuration if it is a [ConfigurationTreeNode].
  #[must_use]
  pub fn as_config(self) -> Option<WickConfiguration> {
    match self {
      ConfigOrDefinition::Config(c) => Some(c.element),
      ConfigOrDefinition::Definition { .. } => None,
    }
  }
  /// Returns the held configuration if it is a [ComponentDefinition].
  #[must_use]
  pub fn as_component_definition(self) -> Option<(String, ComponentDefinition)> {
    match self {
      ConfigOrDefinition::Config(_) => None,
      ConfigOrDefinition::Definition { id, element } => Some((id, element)),
    }
  }
}

#[derive(Debug)]
pub struct ConfigurationTreeNode {
  pub name: String,
  pub element: WickConfiguration,
  pub children: Vec<ConfigOrDefinition>,
}

impl ConfigurationTreeNode {
  #[must_use]
  pub fn new(name: String, element: WickConfiguration) -> Self {
    Self {
      name,
      element,
      children: Vec::new(),
    }
  }

  /// Flattens a configuration tree into a list of its namespaces elements.
  #[must_use]
  pub fn flatten(self) -> Vec<ConfigOrDefinition> {
    let id = self.name.clone();
    flatten(self, &id)
  }

  #[async_recursion::async_recursion]
  pub async fn fetch_children(&mut self, options: FetchOptions) -> Result<(), ManifestError> {
    let root_config = self.element.get_root_config();
    let imports = match &self.element {
      WickConfiguration::Component(c) => c.import.clone(),
      WickConfiguration::App(c) => c.import.clone(),
      _ => Vec::new(),
    };

    let mut children = fetch_imports(imports, options.clone(), root_config).await?;
    for child in children.iter_mut() {
      match child {
        ConfigOrDefinition::Config(c) => {
          c.fetch_children(options.clone()).await?;
        }
        ConfigOrDefinition::Definition { .. } => {}
      }
    }
    self.children = children;

    Ok(())
  }
}

#[must_use]
pub fn flatten(node: ConfigurationTreeNode, prefix: &str) -> Vec<ConfigOrDefinition> {
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

async fn fetch_imports(
  imports: Vec<ImportBinding>,
  options: FetchOptions,
  root_config: Option<&RuntimeConfig>,
) -> Result<Vec<ConfigOrDefinition>, ManifestError> {
  let mut children: Vec<ConfigOrDefinition> = Vec::new();
  let inherent = InherentData::unsafe_default();
  for import in imports {
    let id = import.id().to_owned();

    match import.kind {
      super::ImportDefinition::Component(c) => match c {
        super::ComponentDefinition::Manifest(c) => {
          let mut config = WickConfiguration::fetch(c.reference.clone(), options.clone()).await?;

          let runtime_config = if let Some(c) = c.config() {
            if let Some(value) = &c.value {
              Some(value.clone())
            } else {
              Some(c.render(root_config, None, None, Some(&inherent))?)
            }
          } else {
            None
          };

          config.set_root_config(runtime_config);
          let config = config.finish()?;
          children.push(ConfigOrDefinition::Config(ConfigurationTreeNode::new(id, config)));
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
  use crate::config::{components, AppConfigurationBuilder, ComponentDefinition, ImportBinding, ImportDefinition};

  #[test_logger::test(tokio::test)]
  async fn test_tree_walker() -> Result<()> {
    let mut config = AppConfigurationBuilder::default();

    config
      .name("app")
      .options(FetchOptions::default())
      .import(vec![ImportBinding::new(
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
}
