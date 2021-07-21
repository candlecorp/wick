use std::collections::hash_map::{
  Keys,
  Values,
};
use std::collections::HashMap;
use std::convert::TryFrom;

use vino_manifest::schematic_definition::PortReference;
use vino_provider::ComponentSignature;
use vino_rpc::{
  PortSignature,
  ProviderSignature,
};

use crate::dev::prelude::*;

type Result<T> = std::result::Result<T, SchematicModelError>;

type ComponentId = String;
type ComponentReference = String;
type Namespace = String;

#[derive(Debug, Clone)]
pub(crate) struct SchematicModel {
  definition: SchematicDefinition,
  references: HashMap<ComponentReference, ComponentId>,
  providers: HashMap<Namespace, ProviderModel>,
  upstream_links: HashMap<ConnectionTargetDefinition, ConnectionTargetDefinition>,
  state: Option<LoadedState>,
}

#[derive(Debug, Clone)]
struct LoadedState {
  schematic_inputs: Vec<PortSignature>,
  schematic_outputs: Vec<PortSignature>,
  provider_signatures: Vec<ProviderSignature>,
}

impl TryFrom<SchematicDefinition> for SchematicModel {
  type Error = SchematicModelError;

  fn try_from(definition: SchematicDefinition) -> Result<Self> {
    let references = definition
      .instances
      .iter()
      .map(|(instance, actor)| (instance.clone(), actor.id.clone()))
      .collect();

    let upstream_links = definition
      .connections
      .iter()
      .cloned()
      .map(|connection| (connection.to, connection.from))
      .collect();

    Ok(Self {
      definition,
      references,
      providers: HashMap::new(),
      upstream_links,
      state: None,
    })
  }
}

impl SchematicModel {
  pub(crate) fn get_connections(&self) -> &Vec<ConnectionDefinition> {
    &self.definition.connections
  }

  pub(crate) fn get_component_definitions(&self) -> Values<String, ComponentDefinition> {
    self.definition.instances.values()
  }

  pub(crate) fn get_provider_definitions(&self) -> &Vec<ProviderDefinition> {
    &self.definition.providers
  }

  pub(crate) fn get_references(&self) -> Keys<String, ComponentDefinition> {
    self.definition.instances.keys()
  }

  fn populate_state(&mut self, omit_namespaces: &[String]) -> Result<()> {
    let inputs = self.get_schematic_inputs();
    let mut schematic_inputs = vec![];
    for input in inputs {
      let downstreams = self.get_downstreams(&input);

      let downstream = downstreams.iter().find(|port| {
        let def = self.get_component_definition(port.get_reference());
        match def {
          Some(def) => !omit_namespaces.iter().any(|ns| ns == &def.namespace),
          None => false,
        }
      });
      if downstream.is_none() {
        continue;
      }
      let downstream = downstream.unwrap();
      let model = self
        .get_component_model_by_ref(downstream.get_reference())
        .unwrap();
      let downstream_signature = model
        .inputs
        .iter()
        .find(|port| port.name == downstream.get_port())
        .unwrap();
      schematic_inputs.push(PortSignature {
        name: input.get_port_owned(),
        type_string: downstream_signature.type_string.clone(),
      });
    }
    let outputs = self.get_schematic_outputs();
    let mut schematic_outputs = vec![];
    for output in outputs {
      let upstream = self.get_upstream(&output).unwrap();
      let def = self.get_component_definition(upstream.get_reference());
      if def.is_none() {
        warn!(
          "Reference {} has no component definition",
          upstream.get_reference()
        );
        continue;
      }
      let def = def.unwrap();
      let should_skip = omit_namespaces.iter().any(|ns| ns == &def.namespace);
      if should_skip {
        continue;
      }
      let model = self
        .get_component_model_by_ref(upstream.get_reference())
        .unwrap();
      let downstream_signature = model
        .outputs
        .iter()
        .find(|port| upstream.matches_port(&port.name))
        .unwrap();
      schematic_outputs.push(PortSignature {
        name: output.get_port_owned(),
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

  pub(crate) fn partial_initialization(&mut self) -> Result<()> {
    self.populate_state(&["self".to_owned()])
  }

  pub(crate) fn final_initialization(&mut self) -> Result<()> {
    self.populate_state(&[])
  }

  pub(crate) fn get_upstream(
    &self,
    port: &ConnectionTargetDefinition,
  ) -> Option<&ConnectionTargetDefinition> {
    self.upstream_links.get(port)
  }

  pub(crate) fn get_name(&self) -> String {
    self.definition.get_name()
  }

  pub(crate) fn has_component(&self, id: &str) -> bool {
    let (ns, name) = match parse_id(id) {
      Ok(r) => r,
      Err(_) => return false,
    };
    let provider = self.providers.get(ns);
    provider.map_or(false, |provider| provider.components.get(name).is_some())
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
  pub(crate) fn get_component_model_by_ref(&self, reference: &str) -> Result<ComponentModel> {
    self
      .references
      .get(reference)
      .and_then(|id| self.get_component_model(id))
      .ok_or_else(|| SchematicModelError::MissingComponentModel(reference.to_owned()))
  }

  /// Gets a ComponentModel by component reference string
  pub(crate) fn get_component_model(&self, id: &str) -> Option<ComponentModel> {
    let (ns, name) = match parse_id(id) {
      Ok(result) => result,
      Err(_) => return None,
    };
    let provider = self.providers.get(ns);
    provider.and_then(|provider| provider.components.get(name).cloned())
  }

  /// Gets a ComponentDefinition by component reference string
  pub(crate) fn get_component_definition(&self, reference: &str) -> Option<ComponentDefinition> {
    self.definition.get_component(reference)
  }

  pub(crate) fn get_downstreams(
    &self,
    port: &ConnectionTargetDefinition,
  ) -> Vec<ConnectionTargetDefinition> {
    self
      .definition
      .connections
      .iter()
      .cloned()
      .filter(|conn| &conn.from == port)
      .map(|conn| conn.to)
      .collect()
  }

  pub(crate) fn get_downstream_connections<'a>(
    &'a self,
    reference: &'a str,
  ) -> impl Iterator<Item = &'a ConnectionDefinition> {
    self
      .definition
      .connections
      .iter()
      .filter(move |conn| conn.from.matches_reference(reference))
  }

  pub(crate) fn get_schematic_outputs(&self) -> Vec<ConnectionTargetDefinition> {
    self
      .definition
      .connections
      .iter()
      .cloned()
      .filter(|conn| conn.to.matches_reference(SCHEMATIC_OUTPUT))
      .map(|conn| conn.to)
      .collect()
  }

  pub(crate) fn get_schematic_output_signatures(&self) -> Result<&Vec<PortSignature>> {
    self
      .state
      .as_ref()
      .ok_or(SchematicModelError::ModelNotInitialized)
      .map(|state| &state.schematic_outputs)
  }

  pub(crate) fn get_schematic_inputs(&self) -> Vec<ConnectionTargetDefinition> {
    self
      .definition
      .connections
      .iter()
      .cloned()
      .filter(|conn| conn.from.matches_reference(SCHEMATIC_INPUT))
      .map(|conn| conn.from)
      .collect()
  }

  pub(crate) fn get_schematic_input_signatures(&self) -> Result<&Vec<PortSignature>> {
    self
      .state
      .as_ref()
      .ok_or(SchematicModelError::ModelNotInitialized)
      .map(|state| &state.schematic_inputs)
  }

  pub(crate) fn get_provider_signatures(&self) -> Result<&Vec<ProviderSignature>> {
    self
      .state
      .as_ref()
      .ok_or(SchematicModelError::ModelNotInitialized)
      .map(|state| &state.provider_signatures)
  }

  pub(crate) fn get_outputs(&self, reference: &str) -> Vec<ConnectionTargetDefinition> {
    match self.references.get(reference) {
      Some(id) => match self.get_component_model(id) {
        Some(component) => component
          .outputs
          .iter()
          .map(|p| {
            ConnectionTargetDefinition::from_port(PortReference {
              instance: reference.to_owned(),
              port: p.name.clone(),
            })
          })
          .collect(),
        None => vec![],
      },
      None => vec![],
    }
  }

  // Find the upstream connection for a reference's port
  pub(crate) fn get_upstream_connection<'a>(
    &'a self,
    port: &'a ConnectionTargetDefinition,
  ) -> Option<&'a ConnectionDefinition> {
    self
      .definition
      .connections
      .iter()
      .find(move |connection| &connection.to == port)
  }

  // Find the upstream connections for a reference. Note: this relies on the connections
  // from the definition only, not the component model.
  pub(crate) fn get_upstream_connections_by_reference<'a>(
    &'a self,
    reference: &'a str,
  ) -> impl Iterator<Item = &'a ConnectionDefinition> {
    self
      .definition
      .connections
      .iter()
      .filter(move |connection| connection.to.matches_reference(reference))
  }

  pub(crate) fn get_port_connections<'a>(
    &'a self,
    port: &'a ConnectionTargetDefinition,
  ) -> impl Iterator<Item = &'a ConnectionDefinition> {
    self
      .definition
      .connections
      .iter()
      .filter(move |connection| &connection.from == port || &connection.to == port)
  }

  pub(crate) fn get_defaults(&self) -> impl Iterator<Item = &ConnectionDefinition> {
    self
      .definition
      .connections
      .iter()
      .filter(move |connection| connection.from.is_none() && connection.has_default())
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

  use super::SchematicModel;
  #[allow(unused_imports)]
  use crate::test::prelude::*;

  #[test_env_log::test]
  fn test_basics() -> TestResult<()> {
    let schematic_name = "logger";
    let def = load_schematic_manifest("./src/models/test-schematics/logger.yaml")?;
    let model = SchematicModel::try_from(def)?;
    equals!(model.get_name(), schematic_name);

    Ok(())
  }

  #[test_env_log::test]
  fn test_find_defaults() -> TestResult<()> {
    let schematic_name = "Test";
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition::none(),
      to: ConnectionTargetDefinition::new(SCHEMATIC_OUTPUT, "output"),
      default: Some(serde_json::Value::String("Default string".to_owned())),
    });
    let model = SchematicModel::try_from(schematic_def)?;
    let num = model.get_defaults().count();
    assert_eq!(num, 1);

    Ok(())
  }
}
