use std::collections::HashMap;

use crate::component_model::ComponentModel;
use crate::error::{
  ValidationError,
  VinoError,
};
use crate::provider_model::ProviderModel;
use crate::schematic_definition::*;
use crate::{
  Error,
  PortEntity,
  Result,
  SchematicDefinition,
};

type ComponentReference = String;
type ComponentId = String;
type Namespace = String;

#[derive(Debug, Clone)]
pub struct SchematicModel {
  pub definition: SchematicDefinition,
  pub components: HashMap<ComponentId, ComponentModel>,
  pub references: HashMap<ComponentReference, String>,
  pub providers: HashMap<Namespace, ProviderModel>,
  pub connections: Vec<Connection>,
  pub upstream_links: HashMap<PortEntity, PortEntity>,
}

#[derive(Debug, Clone)]
pub struct Connection {
  pub from: PortEntity,
  pub to: PortEntity,
}

impl SchematicModel {
  pub(crate) fn new(definition: SchematicDefinition) -> Self {
    let references: HashMap<String, String> = definition
      .components
      .iter()
      .map(|(instance, actor)| (instance.to_string(), actor.id.to_string()))
      .collect();
    let connections: Vec<Connection> = definition
      .connections
      .iter()
      .cloned()
      .map(|connection| Connection {
        from: PortEntity::new(connection.from.instance, connection.from.port),
        to: PortEntity::new(connection.to.instance, connection.to.port),
      })
      .collect();
    let upstream_links = connections
      .iter()
      .cloned()
      .map(|connection| (connection.to, connection.from))
      .collect();

    Self {
      definition,
      references,
      connections,
      components: HashMap::new(),
      providers: HashMap::new(),
      upstream_links,
    }
  }
  pub(crate) fn get_upstream(&self, port: &PortEntity) -> Option<&PortEntity> {
    self.upstream_links.get(port)
  }

  pub(crate) fn get_name(&self) -> String {
    self.definition.get_name()
  }
  pub(crate) fn has_namespace(&self, id: &str) -> bool {
    self.providers.contains_key(id)
  }
  pub(crate) fn has_component(&self, id: &str) -> bool {
    let (ns, name) = match parse_namespace(id) {
      Ok(r) => r,
      Err(_) => return false,
    };
    trace!("ns parts: {:?} and {}", ns, name);
    let provider = self.providers.get(&ns);
    if let Some(provider) = provider {
      provider.components.get(&name).is_some()
    } else {
      false
    }
  }
  pub(crate) fn add_provider(&mut self, provider: ProviderModel) -> Result<()> {
    if self.has_namespace(&provider.namespace) {
      Err(Error::SchematicError(format!(
        "Can not add another provider with namespace '{}'",
        provider.namespace
      )))
    } else {
      self.providers.insert(provider.namespace.clone(), provider);
      Ok(())
    }
  }
  /// Gets a ComponentModel by component reference string
  pub(crate) fn get_component_by_ref(&self, reference: &str) -> Option<ComponentModel> {
    self
      .references
      .get(reference)
      .and_then(|id| self.get_component_model(id))
  }
  /// Gets a ComponentModel by component reference string
  pub(crate) fn get_component_model(&self, id: &str) -> Option<ComponentModel> {
    let (ns, name) = match parse_namespace(id) {
      Ok(result) => result,
      Err(_) => return None,
    };
    trace!("ns parts: {:?} and {}", ns, name);
    let provider = self.providers.get(&ns);
    let result = if let Some(provider) = provider {
      provider.components.get(&name).cloned()
    } else {
      None
    };
    trace!("Component model lookup id:{} => {:?}", id, result);
    result
  }
  /// Gets a ComponentDefinition by component reference string
  pub(crate) fn get_component_definition(&self, reference: &str) -> Option<ComponentDefinition> {
    let result = self.definition.get_component(reference);
    trace!(
      "Component definition lookup ref:{} => {:?}",
      reference,
      result
    );
    result
  }

  pub(crate) fn get_downstream_connections(&self, reference: &str) -> Vec<ConnectionDefinition> {
    self
      .definition
      .connections
      .iter()
      .filter(|conn| conn.from.instance == reference)
      .cloned()
      .collect()
  }

  pub(crate) fn get_schematic_outputs(&self) -> Vec<String> {
    self.definition.get_output_names()
  }

  pub(crate) fn get_schematic_inputs(&self) -> Vec<String> {
    self.definition.get_input_names()
  }

  pub(crate) fn get_outputs(&self, reference: &str) -> Vec<String> {
    match self.references.get(reference) {
      Some(id) => match self.get_component_model(id) {
        Some(component) => component.outputs,
        None => vec![],
      },
      None => vec![],
    }
  }
  pub(crate) fn get_connections(&self, reference: &str, port: &str) -> Vec<ConnectionDefinition> {
    let references = &self.references;
    let connections: Vec<ConnectionDefinition> = self
      .definition
      .connections
      .iter()
      .filter(|connection| connection.from.instance == reference && connection.from.port == port)
      .filter_map(|connection| {
        let from_actor = if connection.from.instance == crate::SCHEMATIC_INPUT {
          Some(&connection.from.instance)
        } else {
          references.get(&connection.from.instance)
        };

        let to_actor = if connection.to.instance == crate::SCHEMATIC_OUTPUT {
          Some(&connection.to.instance)
        } else {
          references.get(&connection.to.instance)
        };
        if from_actor.is_none() || to_actor.is_none() {
          return None;
        }
        Some(connection.clone())
      })
      .collect();
    connections
  }
}

type ValidationResult<T> = std::result::Result<T, ValidationError>;
pub(crate) struct Validator<'a> {
  model: &'a SchematicModel,
}

impl<'a> Validator<'a> {
  pub(crate) fn new(model: &'a SchematicModel) -> Self {
    Validator { model }
  }
  pub(crate) fn validate_early_errors(model: &'a SchematicModel) -> Result<()> {
    let validator = Validator::new(model);
    let name = model.get_name();
    let results: Vec<ValidationError> = vec![
      validator.assert_schematic_outputs(),
      validator.assert_schematic_inputs(),
      validator.assert_qualified_names(),
    ]
    .into_iter()
    .filter_map(|r| r.err())
    .collect();
    if results.is_empty() {
      Ok(())
    } else {
      Err(VinoError::ValidationError(ValidationError::EarlyError(
        name, results,
      )))
    }
  }
  fn assert_schematic_outputs(&self) -> ValidationResult<()> {
    let ports = self.model.get_schematic_outputs();
    if ports.is_empty() {
      Err(ValidationError::NoOutputs)
    } else {
      Ok(())
    }
  }
  fn assert_schematic_inputs(&self) -> ValidationResult<()> {
    let ports = self.model.get_schematic_inputs();
    if ports.is_empty() {
      Err(ValidationError::NoInputs)
    } else {
      Ok(())
    }
  }
  fn assert_qualified_names(&self) -> ValidationResult<()> {
    let component_definitions = self.model.components.values();
    let mut errors = vec![];
    for def in component_definitions {
      if parse_namespace(&def.id).is_err() {
        errors.push(def.id.clone());
      }
    }
    if errors.is_empty() {
      Ok(())
    } else {
      Err(ValidationError::NotFullyQualified(errors))
    }
  }
}

#[cfg(test)]
mod tests {

  use std::fs;

  use vino_manifest::{
    Loadable,
    NetworkManifest,
  };

  use super::*;
  use crate::prelude::*;
  use crate::NetworkDefinition;

  fn load_network_manifest(path: &str) -> Result<NetworkDefinition> {
    let manifest = NetworkManifest::V0(vino_manifest::v0::NetworkManifest::from_yaml(
      &fs::read_to_string(path)?,
    )?);
    println!("{:#?}", manifest);
    let def = NetworkDefinition::new(&manifest);
    debug!("Manifest loaded");
    Ok(def)
  }

  #[test_env_log::test]
  fn test_basics() -> Result<()> {
    let schematic_name = "Test";
    let mut schematic_def = SchematicDefinition::new(schematic_name.to_string());
    schematic_def.providers.push(ProviderDefinition {
      namespace: "test-namespace".to_string(),
      kind: ProviderKind::Native,
      reference: "internal".to_string(),
      data: HashMap::new(),
    });
    schematic_def.components.insert(
      "logger".to_string(),
      ComponentDefinition {
        metadata: None,
        id: "test-namespace::log".to_string(),
      },
    );
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: SCHEMATIC_INPUT.to_string(),
        port: "input".to_string(),
      },
      to: ConnectionTargetDefinition {
        instance: "logger".to_string(),
        port: "input".to_string(),
      },
    });
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "logger".to_string(),
        port: "output".to_string(),
      },
      to: ConnectionTargetDefinition {
        instance: SCHEMATIC_OUTPUT.to_string(),
        port: "output".to_string(),
      },
    });
    let model = SchematicModel::new(schematic_def);
    assert_eq!(model.get_name(), schematic_name);
    println!("{:?}", model);

    Ok(())
  }

  #[test_env_log::test]
  fn test_validate_early_errors() -> Result<()> {
    let def = load_network_manifest("./manifests/native-component.yaml")?;
    let model = SchematicModel::new(def.schematics[0].clone());

    println!("{:?}", model);
    Validator::validate_early_errors(&model)?;
    Ok(())
  }

  #[test_env_log::test]
  fn test_connections() -> Result<()> {
    let schematic_name = "Test";
    let mut schematic_def = SchematicDefinition::new(schematic_name.to_string());
    schematic_def.providers.push(ProviderDefinition {
      namespace: "test-namespace".to_string(),
      kind: ProviderKind::Native,
      reference: "internal".to_string(),
      data: HashMap::new(),
    });
    schematic_def.components.insert(
      "logger".to_string(),
      ComponentDefinition {
        metadata: None,
        id: "test-namespace::log".to_string(),
      },
    );
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: SCHEMATIC_INPUT.to_string(),
        port: "input".to_string(),
      },
      to: ConnectionTargetDefinition {
        instance: "logger".to_string(),
        port: "input".to_string(),
      },
    });
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "logger".to_string(),
        port: "output".to_string(),
      },
      to: ConnectionTargetDefinition {
        instance: SCHEMATIC_OUTPUT.to_string(),
        port: "output".to_string(),
      },
    });
    let model = SchematicModel::new(schematic_def);
    assert_eq!(model.get_name(), schematic_name);

    let upstream = model
      .get_upstream(&PortEntity::new("logger".to_string(), "input".to_string()))
      .unwrap();
    assert_eq!(upstream.reference, SCHEMATIC_INPUT);
    assert_eq!(upstream.name, "input");

    println!("{:?}", model);

    Ok(())
  }
}
