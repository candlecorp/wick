use std::collections::HashMap;
use std::fmt::Display;

use vino_provider::ComponentSignature;
use vino_rpc::{
  PortSignature,
  ProviderSignature,
};

use crate::dev::prelude::*;
use crate::error::{
  SchematicError,
  ValidationError,
};

type Result<T> = std::result::Result<T, SchematicError>;

type ComponentReference = String;
type Namespace = String;

#[derive(Debug, Clone)]
pub(crate) struct SchematicModel {
  definition: SchematicDefinition,
  references: HashMap<ComponentReference, String>,
  providers: HashMap<Namespace, ProviderModel>,
  pub(crate) connections: Vec<Connection>,
  upstream_links: HashMap<PortReference, PortReference>,
  state: Option<LoadedState>,
}

#[derive(Debug, Clone)]
struct LoadedState {
  schematic_inputs: Vec<PortSignature>,
  schematic_outputs: Vec<PortSignature>,
  provider_signatures: Vec<ProviderSignature>,
}

#[derive(Debug, Clone)]
pub(crate) struct Connection {
  pub(crate) from: PortReference,
  pub(crate) to: PortReference,
}

impl From<ConnectionDefinition> for Connection {
  fn from(v: ConnectionDefinition) -> Self {
    Connection {
      from: v.from.into(),
      to: v.to.into(),
    }
  }
}

impl Display for Connection {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} => {}", self.from, self.to)
  }
}

impl From<vino_manifest::v0::ConnectionTargetDefinition> for PortReference {
  fn from(def: vino_manifest::v0::ConnectionTargetDefinition) -> Self {
    PortReference {
      name: def.port,
      reference: def.instance,
    }
  }
}

impl Connection {
  pub(crate) fn print_all(list: &[Self]) -> String {
    list
      .iter()
      .map(std::string::ToString::to_string)
      .collect::<Vec<String>>()
      .join(", ")
  }
}

impl SchematicModel {
  pub(crate) fn new(definition: SchematicDefinition) -> Self {
    let references: HashMap<String, String> = definition
      .components
      .iter()
      .map(|(instance, actor)| (instance.clone(), actor.id.clone()))
      .collect();
    let connections: Vec<Connection> = definition
      .connections
      .iter()
      .cloned()
      .map(From::from)
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
      providers: HashMap::new(),
      upstream_links,
      state: None,
    }
  }

  pub(crate) fn finish_initialization(&mut self) -> Result<()> {
    // These are safe because we finish initializing after
    // validating schematic model is sound.
    let inputs = self.get_schematic_inputs();
    let mut schematic_inputs = vec![];
    for input in inputs {
      let downstreams = self.get_downstreams(&input);
      let downstream = &downstreams[0];
      let model = self
        .get_component_model_by_ref(&downstream.reference)
        .unwrap();
      let downstream_signature = model
        .inputs
        .iter()
        .find(|port| port.name == downstream.name)
        .unwrap();
      schematic_inputs.push(PortSignature {
        name: input.name,
        type_string: downstream_signature.type_string.clone(),
      });
    }
    let outputs = self.get_schematic_outputs();
    let mut schematic_outputs = vec![];
    for output in outputs {
      let upstream = self.get_upstream(&output).unwrap();
      let model = self
        .get_component_model_by_ref(&upstream.reference)
        .unwrap();
      let downstream_signature = model
        .outputs
        .iter()
        .find(|port| port.name == upstream.name)
        .unwrap();
      schematic_outputs.push(PortSignature {
        name: output.name,
        type_string: downstream_signature.type_string.clone(),
      });
    }
    let provider_signatures = self
      .providers
      .iter()
      .map(|(ns, provider_model)| ProviderSignature {
        name: ns.clone(),
        components: provider_model
          .components
          .values()
          .map(|model| model.into())
          .collect(),
      })
      .collect();
    self.state = Some(LoadedState {
      provider_signatures,
      schematic_inputs,
      schematic_outputs,
    });
    Ok(())
  }

  pub(crate) fn get_upstream(&self, port: &PortReference) -> Option<&PortReference> {
    self.upstream_links.get(port)
  }

  pub(crate) fn get_name(&self) -> String {
    self.definition.get_name()
  }

  pub(crate) fn has_component(&self, id: &str) -> bool {
    let (ns, name) = match parse_namespace(id) {
      Ok(r) => r,
      Err(_) => return false,
    };
    trace!("ns parts: {:?} and {}", ns, name);
    let provider = self.providers.get(&ns);
    provider.map_or(false, |provider| provider.components.get(&name).is_some())
  }
  pub(crate) fn commit_providers(&mut self, providers: Vec<ProviderModel>) {
    self.providers = providers
      .into_iter()
      .map(|p| (p.namespace.clone(), p))
      .collect();
  }

  /// Gets a ComponentModel by component reference string
  pub(crate) fn get_component_model_by_ref(&self, reference: &str) -> Option<ComponentModel> {
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
    let result = provider.and_then(|provider| provider.components.get(&name).cloned());
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

  pub(crate) fn get_downstreams(&self, port: &PortReference) -> Vec<PortReference> {
    self
      .definition
      .connections
      .iter()
      .cloned()
      .filter(|conn| conn.from.instance == port.reference && conn.from.port == port.name)
      .map(|conn| conn.to.into())
      .collect()
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

  pub(crate) fn get_schematic_outputs(&self) -> Vec<PortReference> {
    self
      .definition
      .connections
      .iter()
      .cloned()
      .filter(|conn| conn.to.instance == SCHEMATIC_OUTPUT)
      .map(|conn| conn.to.into())
      .collect()
  }

  pub(crate) fn get_schematic_output_signatures(&self) -> Result<&Vec<PortSignature>> {
    self
      .state
      .as_ref()
      .ok_or(SchematicError::ModelNotInitialized)
      .map(|state| &state.schematic_outputs)
  }

  pub(crate) fn get_schematic_inputs(&self) -> Vec<PortReference> {
    self
      .definition
      .connections
      .iter()
      .cloned()
      .filter(|conn| conn.from.instance == SCHEMATIC_INPUT)
      .map(|conn| conn.from.into())
      .collect()
  }

  pub(crate) fn get_schematic_input_signatures(&self) -> Result<&Vec<PortSignature>> {
    self
      .state
      .as_ref()
      .ok_or(SchematicError::ModelNotInitialized)
      .map(|state| &state.schematic_inputs)
  }

  pub(crate) fn get_provider_signatures(&self) -> Result<&Vec<ProviderSignature>> {
    self
      .state
      .as_ref()
      .ok_or(SchematicError::ModelNotInitialized)
      .map(|state| &state.provider_signatures)
  }

  pub(crate) fn get_outputs(&self, reference: &str) -> Vec<PortReference> {
    match self.references.get(reference) {
      Some(id) => match self.get_component_model(id) {
        Some(component) => component
          .outputs
          .iter()
          .map(|p| PortReference {
            reference: reference.to_owned(),
            name: p.name.clone(),
          })
          .collect(),
        None => vec![],
      },
      None => vec![],
    }
  }
  pub(crate) fn get_connections(&self, port: &PortReference) -> &Vec<Connection> {
    &self.connections
  }
}

impl From<&ComponentModel> for ComponentSignature {
  fn from(v: &ComponentModel) -> Self {
    ComponentSignature {
      name: v.name.clone(),
      inputs: v.inputs.clone(),
      outputs: v.outputs.clone(),
    }
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
  pub(crate) fn validate_early_errors(
    model: &'a SchematicModel,
  ) -> std::result::Result<(), ValidationError> {
    let validator = Validator::new(model);
    let name = model.get_name();
    let results: Vec<ValidationError> = vec![
      validator.assert_early_schematic_inputs(),
      validator.assert_early_schematic_outputs(),
      validator.assert_early_qualified_names(),
      validator.assert_no_dangling_references(),
    ]
    .into_iter()
    .filter_map(std::result::Result::err)
    .collect();
    if results.is_empty() {
      Ok(())
    } else {
      Err(ValidationError::EarlyError(name, results))
    }
  }
  pub(crate) fn validate_late_errors(
    model: &'a SchematicModel,
  ) -> std::result::Result<(), ValidationError> {
    let validator = Validator::new(model);
    let name = model.get_name();
    let results: Vec<ValidationError> = vec![validator.assert_component_models()]
      .into_iter()
      .filter_map(std::result::Result::err)
      .collect();
    if results.is_empty() {
      Ok(())
    } else {
      Err(ValidationError::Error(name, results))
    }
  }
  pub(crate) fn validate(model: &'a SchematicModel) -> std::result::Result<(), ValidationError> {
    Self::validate_early_errors(model)?;
    Self::validate_late_errors(model)
  }
  fn assert_no_dangling_references(&self) -> ValidationResult<()> {
    let dangling: Vec<String> = self
      .model
      .definition
      .connections
      .iter()
      .flat_map(|conn| {
        let from = self.model.get_component_definition(&conn.from.instance);
        let to = self.model.get_component_definition(&conn.to.instance);
        let mut none = vec![];
        if from.is_none() && conn.from.instance != SCHEMATIC_INPUT {
          none.push(Some(conn.from.instance.clone()));
        }
        if to.is_none() && conn.to.instance != SCHEMATIC_OUTPUT {
          none.push(Some(conn.to.instance.clone()));
        }
        none
      })
      .flatten()
      .collect();
    if dangling.is_empty() {
      Ok(())
    } else {
      Err(ValidationError::DanglingReference(dangling))
    }
  }
  fn assert_component_models(&self) -> ValidationResult<()> {
    let missing_components: Vec<String> = self
      .model
      .definition
      .components
      .keys()
      .filter_map(|r| {
        let def = self.model.get_component_definition(r).unwrap();
        let model = self.model.get_component_model(&def.id);
        model
          .is_none()
          .then(|| format!("{}=>{}", r.clone(), def.id))
      })
      .collect();

    if missing_components.is_empty() {
      Ok(())
    } else {
      Err(ValidationError::MissingComponentModels(missing_components))
    }
  }
  fn assert_early_schematic_outputs(&self) -> ValidationResult<()> {
    let ports = self.model.get_schematic_outputs();
    if ports.is_empty() {
      Err(ValidationError::NoOutputs)
    } else {
      Ok(())
    }
  }
  fn assert_early_schematic_inputs(&self) -> ValidationResult<()> {
    let ports = self.model.get_schematic_inputs();
    if ports.is_empty() {
      Err(ValidationError::NoInputs)
    } else {
      Ok(())
    }
  }
  fn assert_early_qualified_names(&self) -> ValidationResult<()> {
    let component_definitions = self.model.definition.components.values();
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

  use std::collections::HashMap;
  use std::fs;

  use vino_manifest::{
    Loadable,
    NetworkManifest,
  };
  use vino_provider::ComponentSignature;
  use vino_rpc::{
    PortSignature,
    ProviderSignature,
  };

  use super::{
    SchematicModel,
    Validator,
  };
  use crate::error::ValidationError;
  #[allow(unused_imports)]
  use crate::test::prelude::*;

  fn load_network_manifest(path: &str) -> Result<NetworkDefinition> {
    let manifest = NetworkManifest::V0(vino_manifest::v0::NetworkManifest::from_yaml(
      &fs::read_to_string(path)?,
    )?);
    let def = NetworkDefinition::new(&manifest);
    debug!("Manifest loaded");
    Ok(def)
  }

  #[test_env_log::test]
  fn test_basics() -> Result<()> {
    let schematic_name = "Test";
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.providers.push(ProviderDefinition {
      namespace: "test-namespace".to_owned(),
      kind: ProviderKind::Native,
      reference: "internal".to_owned(),
      data: HashMap::new(),
    });
    schematic_def.components.insert(
      "logger".to_owned(),
      ComponentDefinition {
        metadata: None,
        id: "test-namespace::log".to_owned(),
      },
    );
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: SCHEMATIC_INPUT.to_owned(),
        port: "input".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: "logger".to_owned(),
        port: "input".to_owned(),
      },
    });
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "logger".to_owned(),
        port: "output".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: SCHEMATIC_OUTPUT.to_owned(),
        port: "output".to_owned(),
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
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.providers.push(ProviderDefinition {
      namespace: "test-namespace".to_owned(),
      kind: ProviderKind::Native,
      reference: "internal".to_owned(),
      data: HashMap::new(),
    });
    schematic_def.components.insert(
      "logger".to_owned(),
      ComponentDefinition {
        metadata: None,
        id: "test-namespace::log".to_owned(),
      },
    );
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: SCHEMATIC_INPUT.to_owned(),
        port: "input".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: "logger".to_owned(),
        port: "input".to_owned(),
      },
    });
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "logger".to_owned(),
        port: "output".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: SCHEMATIC_OUTPUT.to_owned(),
        port: "output".to_owned(),
      },
    });
    let model = SchematicModel::new(schematic_def);
    assert_eq!(model.get_name(), schematic_name);

    let upstream = model
      .get_upstream(&PortReference::new("logger".to_owned(), "input".to_owned()))
      .unwrap();
    assert_eq!(upstream.reference, SCHEMATIC_INPUT);
    assert_eq!(upstream.name, "input");

    println!("{:?}", model);

    Ok(())
  }

  #[test_env_log::test]
  fn test_dangling_refs() -> Result<()> {
    let schematic_name = "Test";
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "dangling1".to_owned(),
        port: "output".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: SCHEMATIC_OUTPUT.to_owned(),
        port: "output".to_owned(),
      },
    });
    let model = SchematicModel::new(schematic_def);
    assert_eq!(model.get_name(), schematic_name);
    let result = Validator::validate_early_errors(&model);
    assert_eq!(
      result,
      Err(ValidationError::EarlyError(
        "Test".to_owned(),
        vec![
          ValidationError::NoInputs,
          ValidationError::DanglingReference(vec!["dangling1".to_owned()]),
        ]
      ))
    );

    Ok(())
  }

  #[test_env_log::test]
  fn test_missing_models() -> Result<()> {
    let schematic_name = "Test";
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.components.insert(
      "logger".to_owned(),
      ComponentDefinition {
        metadata: None,
        id: "test-namespace::log".to_owned(),
      },
    );
    let model = SchematicModel::new(schematic_def);
    let result = Validator::validate_late_errors(&model);
    assert_eq!(
      result,
      Err(ValidationError::Error(
        "Test".to_owned(),
        vec![ValidationError::MissingComponentModels(vec![
          "logger=>test-namespace::log".to_owned()
        ]),]
      ))
    );
    println!("{:?}", model);

    Ok(())
  }

  #[test_env_log::test]
  fn test_finish_initialization() -> Result<()> {
    let schematic_name = "Test";
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.providers.push(ProviderDefinition {
      namespace: "test-namespace".to_owned(),
      kind: ProviderKind::Native,
      reference: "internal".to_owned(),
      data: HashMap::new(),
    });
    schematic_def.components.insert(
      "logger".to_owned(),
      ComponentDefinition {
        metadata: None,
        id: "test-namespace::log".to_owned(),
      },
    );
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: SCHEMATIC_INPUT.to_owned(),
        port: "input".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: "logger".to_owned(),
        port: "input".to_owned(),
      },
    });
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "logger".to_owned(),
        port: "output".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: SCHEMATIC_OUTPUT.to_owned(),
        port: "output".to_owned(),
      },
    });
    let mut model = SchematicModel::new(schematic_def);
    let provider = ProviderModel {
      namespace: "test-namespace".to_owned(),
      components: hashmap! {
        "log".to_owned() => ComponentModel {
          id: "test-namespace::log".to_owned(),
          name: "log".to_owned(),
          inputs: vec![PortSignature{name: "input".to_owned(), type_string: "string".to_owned()}],
          outputs: vec![PortSignature{name: "output".to_owned(), type_string: "bytes".to_owned()}],
        }
      },
    };
    model.commit_providers(vec![provider]);
    let result = Validator::validate(&model);
    assert_eq!(result, Ok(()));
    model.finish_initialization()?;
    let schematic_inputs = model.get_schematic_input_signatures()?;
    assert_eq!(
      schematic_inputs,
      &vec![PortSignature {
        name: "input".to_owned(),
        type_string: "string".to_owned()
      }]
    );
    let schematic_outputs = model.get_schematic_output_signatures()?;
    assert_eq!(
      schematic_outputs,
      &vec![PortSignature {
        name: "output".to_owned(),
        type_string: "bytes".to_owned()
      }]
    );
    let provider_sigs = model.get_provider_signatures()?;
    assert_eq!(provider_sigs.len(), 1);
    assert_eq!(
      provider_sigs[0],
      ProviderSignature {
        name: "test-namespace".to_owned(),
        components: vec![ComponentSignature {
          name: "log".to_owned(),
          inputs: vec![PortSignature {
            name: "input".to_owned(),
            type_string: "string".to_owned()
          }],
          outputs: vec![PortSignature {
            name: "output".to_owned(),
            type_string: "bytes".to_owned()
          }]
        }]
      }
    );

    Ok(())
  }
}
