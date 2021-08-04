use std::collections::hash_map::{
  Keys,
  Values,
};
use std::collections::HashMap;
use std::convert::TryFrom;

use vino_manifest::schematic_definition::PortReference;
use vino_provider::native::prelude::*;

use crate::dev::prelude::*;

type Result<T> = std::result::Result<T, SchematicModelError>;

type ComponentId = String;
type ComponentInstance = String;
type Namespace = String;

#[derive(Debug, Clone)]
pub(crate) struct SchematicModel {
  definition: SchematicDefinition,
  instances: HashMap<ComponentInstance, ComponentId>,
  providers: HashMap<Namespace, ProviderModel>,
  upstream_links: HashMap<ConnectionTargetDefinition, ConnectionTargetDefinition>,
  state: Option<LoadedState>,
  raw_ports: HashMap<String, RawPorts>,
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
    let instances = definition
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

    let raw_ports = get_raw_ports(&definition)?;

    Ok(Self {
      definition,
      instances,
      providers: HashMap::new(),
      upstream_links,
      state: None,
      raw_ports,
    })
  }
}

pub(crate) fn get_raw_ports(def: &SchematicDefinition) -> Result<HashMap<String, RawPorts>> {
  let mut map = HashMap::new();
  for connection in &def.connections {
    let from = connection.from.get_instance_owned();
    let ports = map.entry(from).or_insert_with(RawPorts::default);
    ports.outputs.insert(connection.from.clone());
    let to = connection.to.get_instance_owned();
    let ports = map.entry(to).or_insert_with(RawPorts::default);
    ports.inputs.insert(connection.to.clone());
  }
  Ok(map)
}

impl SchematicModel {
  pub(crate) fn get_connections(&self) -> &Vec<ConnectionDefinition> {
    &self.definition.connections
  }

  pub(crate) fn get_component_definitions(&self) -> Values<String, ComponentDefinition> {
    self.definition.instances.values()
  }

  pub(crate) fn get_instances(&self) -> Keys<String, ComponentDefinition> {
    self.definition.instances.keys()
  }

  pub(crate) fn get_raw_ports(&self) -> &HashMap<String, RawPorts> {
    &self.raw_ports
  }

  fn populate_state(&mut self, omit_namespaces: &[String]) -> Result<()> {
    let inputs = self.get_schematic_inputs();
    let mut input_signatures = vec![];
    let should_skip_namespace = |namespace: &str| omit_namespaces.iter().any(|ns| ns == namespace);

    for input in inputs {
      // This loop grabs the first valid connection for each schematic
      // input and assumes its type is the type of the input. This is true for now
      // but that may not hold forever.
      let to_ports = self.get_downstreams(&input);

      let downstream = to_ports.iter().find(|port| {
        let instance_id = port.get_instance();
        let def = self.get_component_definition(instance_id);
        match def {
          Some(def) => !should_skip_namespace(&def.namespace),
          None => false,
        }
      });
      let downstream = some_or_continue!(downstream);
      let downstream_instance = downstream.get_instance();

      let model = match self.get_component_model_by_instance(downstream_instance) {
        Some(model) => model,
        None => {
          debug!("{} does not have valid model.", downstream_instance);
          continue;
        }
      };
      let downstream_port = downstream.get_port();

      let downstream_signature = model
        .inputs
        .iter()
        .find(|port| port.name == downstream_port);

      let downstream_signature = match downstream_signature {
        Some(d) => d,
        None => {
          debug!(
            "Model {:?} does not have expected port {}",
            model, downstream_port
          );
          continue;
        }
      };

      input_signatures.push(PortSignature {
        name: input.get_port_owned(),
        type_string: downstream_signature.type_string.clone(),
      });
    }
    let outputs = self.get_schematic_outputs();
    let mut output_signatures = vec![];
    for output in outputs {
      let opt = self
        .get_upstream(output)
        .and_then(|upstream| {
          self
            .get_component_model_by_instance(upstream.get_instance())
            .map(|model| (upstream, model))
        })
        .and_then(|(upstream, model)| {
          if should_skip_namespace(&model.namespace) {
            return None;
          }
          model
            .outputs
            .iter()
            .find(|port| upstream.matches_port(&port.name))
            .map(|signature| signature.type_string.clone())
        });
      let signature = match opt {
        Some(sig) => sig,
        None => {
          warn!("Could not find signature for output '{}'", output);
          continue;
        }
      };

      output_signatures.push(PortSignature {
        name: output.get_port_owned(),
        type_string: signature,
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
      schematic_inputs: input_signatures,
      schematic_outputs: output_signatures,
    });
    Ok(())
  }

  pub(crate) fn partial_initialization(&mut self) -> Result<()> {
    self.populate_state(&["self".to_owned()])
  }

  pub(crate) fn final_initialization(&mut self) -> Result<()> {
    self.populate_state(&[])
  }

  pub(crate) fn get_upstreams(
    &self,
  ) -> &HashMap<ConnectionTargetDefinition, ConnectionTargetDefinition> {
    &self.upstream_links
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
    trace!(
      "SC:{}:PROVIDERS:[{}]",
      self.get_name(),
      providers
        .iter()
        .map(|p| p.namespace.clone())
        .collect::<Vec<String>>()
        .join(", "),
    );
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

  /// Gets a ComponentModel by component instance string
  pub(crate) fn get_component_model_by_instance(&self, instance: &str) -> Option<ComponentModel> {
    self
      .instances
      .get(instance)
      .and_then(|id| self.get_component_model(id))
  }

  /// Gets a ComponentModel by component id
  pub(crate) fn get_component_model(&self, id: &str) -> Option<ComponentModel> {
    let (ns, name) = match parse_id(id) {
      Ok(result) => result,
      Err(_) => return None,
    };
    let provider = self.providers.get(ns);
    provider.and_then(|provider| provider.components.get(name).cloned())
  }

  /// Gets a ComponentDefinition by component instance string
  pub(crate) fn get_component_definition(&self, instance: &str) -> Option<ComponentDefinition> {
    self.definition.get_component(instance)
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
    instance: &'a str,
  ) -> impl Iterator<Item = &'a ConnectionDefinition> {
    self
      .definition
      .connections
      .iter()
      .filter(move |conn| conn.from.matches_instance(instance))
  }

  pub(crate) fn get_schematic_outputs(&self) -> impl Iterator<Item = &ConnectionTargetDefinition> {
    self
      .definition
      .connections
      .iter()
      .filter(|conn| conn.to.matches_instance(SCHEMATIC_OUTPUT))
      .map(|conn| &conn.to)
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
      .filter(|conn| conn.from.matches_instance(SCHEMATIC_INPUT))
      .map(|conn| conn.from)
      .collect()
  }

  pub(crate) fn is_generator(&self, instance: &str) -> bool {
    if instance == SCHEMATIC_INPUT {
      false
    } else {
      self
        .get_raw_ports()
        .get(instance)
        .map_or(false, |rp| rp.inputs.is_empty())
    }
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

  pub(crate) fn get_outputs(&self, instance: &str) -> Vec<ConnectionTargetDefinition> {
    match self.instances.get(instance) {
      Some(id) => match self.get_component_model(id) {
        Some(component) => component
          .outputs
          .iter()
          .map(|p| {
            ConnectionTargetDefinition::from_port(PortReference {
              instance: instance.to_owned(),
              port: p.name.clone(),
            })
          })
          .collect(),
        None => vec![],
      },
      None => vec![],
    }
  }

  // Find the upstream connection for a instance's port
  #[allow(unused)]
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

  // Find the upstream connections for a instance. Note: this relies on the connections
  // from the definition only, not the component model.
  pub(crate) fn _get_upstream_connections_by_instance<'a>(
    &'a self,
    instance: &'a str,
  ) -> impl Iterator<Item = &'a ConnectionDefinition> {
    self
      .definition
      .connections
      .iter()
      .filter(move |connection| connection.to.matches_instance(instance))
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

  pub(crate) fn get_senders(&self) -> impl Iterator<Item = &ConnectionDefinition> {
    self
      .definition
      .connections
      .iter()
      .filter(move |connection| connection.from.is_sender())
  }
  pub(crate) fn get_generators(&self) -> impl Iterator<Item = &ConnectionDefinition> {
    self
      .definition
      .connections
      .iter()
      .filter(move |connection| self.is_generator(connection.from.get_instance()))
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

  #[allow(unused_imports)]
  use crate::test::prelude::{
    assert_eq,
    *,
  };
  #[test_env_log::test]
  fn test_basics() -> TestResult<()> {
    let schematic_name = "logger";
    let def = load_schematic_manifest("./src/models/test-schematics/logger.yaml")?;
    let model = SchematicModel::try_from(def)?;
    assert_eq!(model.get_name(), schematic_name);

    Ok(())
  }

  #[test_env_log::test]
  fn test_find_defaults() -> TestResult<()> {
    let schematic_name = "Test";
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition::sender(None),
      to: ConnectionTargetDefinition::new(SCHEMATIC_OUTPUT, "output"),
      default: Some(serde_json::Value::String("Default string".to_owned())),
    });
    let model = SchematicModel::try_from(schematic_def)?;
    let num = model.get_senders().count();
    assert_eq!(num, 1);

    Ok(())
  }
}
