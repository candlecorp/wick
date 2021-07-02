use std::collections::hash_map::{
  Keys,
  Values,
};
use std::collections::HashMap;
use std::fmt::Display;

use vino_provider::ComponentSignature;
use vino_rpc::{
  PortSignature,
  ProviderSignature,
};

use crate::dev::prelude::*;
use crate::error::SchematicError;

type Result<T> = std::result::Result<T, SchematicError>;

type ComponentReference = String;
type Namespace = String;

#[derive(Debug, Clone)]
pub(crate) struct SchematicModel {
  definition: SchematicDefinition,
  references: HashMap<ComponentReference, String>,
  providers: HashMap<Namespace, ProviderModel>,
  connections: Vec<Connection>,
  upstream_links: HashMap<PortReference, PortReference>,
  state: Option<LoadedState>,
}

#[derive(Debug, Clone)]
struct LoadedState {
  schematic_inputs: Vec<PortSignature>,
  schematic_outputs: Vec<PortSignature>,
  provider_signatures: Vec<ProviderSignature>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Connection {
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

  pub(crate) fn get_definition(&self) -> &SchematicDefinition {
    &self.definition
  }

  pub(crate) fn get_connections(&self) -> &Vec<Connection> {
    &self.connections
  }

  pub(crate) fn get_component_definitions(&self) -> Values<String, ComponentDefinition> {
    self.definition.components.values()
  }

  pub(crate) fn get_references(&self) -> Keys<String, ComponentDefinition> {
    self.definition.components.keys()
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
      trace!(
        "input: {}, downstream: {:?}, model: {:?}",
        input,
        downstream,
        model
      );
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
    let provider = self.providers.get(&ns);
    provider.map_or(false, |provider| provider.components.get(&name).is_some())
  }

  pub(crate) fn commit_providers(&mut self, providers: Vec<ProviderModel>) {
    self.providers = providers
      .into_iter()
      .map(|p| (p.namespace.clone(), p))
      .collect();
    // ensure state is reset;
    self.state = None;
  }

  pub(crate) fn commit_self_provider(&mut self, provider: ProviderModel) {
    self.providers.insert("self".to_owned(), provider);
    // ensure state is reset;
    self.state = None;
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
    let provider = self.providers.get(&ns);
    let result = provider.and_then(|provider| provider.components.get(&name).cloned());
    testlog!("Component model lookup id:{} => {:?}", id, result);
    result
  }

  /// Gets a ComponentDefinition by component reference string
  pub(crate) fn get_component_definition(&self, reference: &str) -> Option<ComponentDefinition> {
    let result = self.definition.get_component(reference);
    testlog!(
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
  pub(crate) fn get_port_connections(&self, port: &PortReference) -> Vec<Connection> {
    self
      .connections
      .iter()
      .cloned()
      .filter(|connection| &connection.from == port || &connection.to == port)
      .collect()
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

#[cfg(test)]
mod tests {

  use std::collections::HashMap;

  use super::SchematicModel;
  #[allow(unused_imports)]
  use crate::test::prelude::*;

  #[test_env_log::test]
  fn test_basics() -> Result<(), SchematicError> {
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
      ComponentDefinition::new("test-namespace", "log"),
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
    equals!(model.get_name(), schematic_name);

    Ok(())
  }
}
