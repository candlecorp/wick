use std::collections::hash_map::{
  Keys,
  Values,
};
use std::collections::HashMap;
use std::convert::TryFrom;

use vino_manifest::schematic_definition::PortReference;
use vino_provider::native::prelude::*;

use crate::dev::prelude::*;
use crate::VINO_V0_NAMESPACE;

type Result<T> = std::result::Result<T, SchematicModelError>;

type ComponentId = String;
type ComponentInstance = String;
type Namespace = String;

#[derive(Debug, Clone)]
pub(crate) struct SchematicModel {
  definition: SchematicDefinition,
  instances: HashMap<ComponentInstance, ComponentId>,
  providers: HashMap<Namespace, Option<ProviderModel>>,
  upstream_links: HashMap<ConnectionTargetDefinition, ConnectionTargetDefinition>,
  signature: Option<SchematicSignature>,
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

  fn try_from(mut definition: SchematicDefinition) -> Result<Self> {
    let instances = definition
      .instances
      .iter()
      .map(|(instance, component)| (instance.clone(), component.id()))
      .collect();

    let upstream_links = definition
      .connections
      .iter()
      .cloned()
      .map(|connection| (connection.to, connection.from))
      .collect();

    if !definition.providers.contains(&SELF_NAMESPACE.to_owned()) {
      definition.providers.push(SELF_NAMESPACE.to_owned());
    }

    if !definition.providers.contains(&VINO_V0_NAMESPACE.to_owned()) {
      definition.providers.push(VINO_V0_NAMESPACE.to_owned());
    }

    let raw_ports = get_raw_ports(&definition)?;

    Ok(Self {
      definition,
      instances,
      providers: HashMap::new(),
      upstream_links,
      signature: None,
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

  fn populate_signature(&mut self, omit_namespaces: &[String]) -> Result<()> {
    let provider_signatures = self.build_provider_signatures();
    self.signature = Some(SchematicSignature {
      name: self.get_name(),
      providers: provider_signatures,
      inputs: vec![],
      outputs: vec![],
    });
    let input_signatures = self.infer_schematic_inputs(omit_namespaces)?;
    let output_signatures = self.infer_schematic_outputs(omit_namespaces)?;
    let sig = self.signature.as_mut().unwrap();
    sig.inputs = input_signatures;
    sig.outputs = output_signatures;
    Ok(())
  }

  fn infer_schematic_inputs(&self, omit_namespaces: &[String]) -> Result<Vec<PortSignature>> {
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
        Some((_, model)) => model,
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
    Ok(input_signatures)
  }

  fn infer_schematic_outputs(&self, omit_namespaces: &[String]) -> Result<Vec<PortSignature>> {
    let outputs = self.get_schematic_outputs();
    let mut output_signatures = vec![];
    let should_skip_namespace = |namespace: &str| omit_namespaces.iter().any(|ns| ns == namespace);

    for output in outputs {
      let opt = self
        .get_upstream(output)
        .and_then(|upstream| {
          self
            .get_component_model_by_instance(upstream.get_instance())
            .map(|model| (upstream, model))
        })
        .and_then(|(upstream, (ns, model))| {
          if should_skip_namespace(&ns) {
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
          logtest!("Could not find signature for output '{}'", output);
          continue;
        }
      };

      output_signatures.push(PortSignature {
        name: output.get_port_owned(),
        type_string: signature,
      });
    }
    Ok(output_signatures)
  }

  fn build_provider_signatures(&self) -> Vec<ProviderSignature> {
    let provider_signatures = self
      .providers
      .iter()
      .filter_map(|(ns, provider_model)| {
        provider_model.as_ref().map(|model| ProviderSignature {
          name: ns.clone(),
          components: model
            .components
            .values()
            .map(|model| model.into())
            .collect(),
        })
      })
      .collect();
    provider_signatures
  }

  // TODO: assess
  #[allow(unused)]
  pub(crate) fn partial_initialization(&mut self) -> Result<()> {
    trace!("MODEL:SC[{}]:PARTIAL_INIT", self.get_name());
    self.populate_signature(&["self".to_owned()])
  }

  pub(crate) fn finalize(&mut self) -> Result<()> {
    trace!("MODEL:SC[{}]:FINAL_INIT", self.get_name());
    self.populate_signature(&[])
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

  pub(crate) fn has_component(&self, component: &ComponentDefinition) -> bool {
    let name = &component.name;
    match self.providers.get(&component.namespace) {
      Some(Some(provider)) => provider.components.get(name).is_some(),
      _ => false,
    }
  }

  #[allow(clippy::ptr_arg)]
  pub(crate) fn is_provider_allowed(&self, namespace: &String) -> bool {
    self.definition.providers.contains(namespace)
  }

  pub(crate) fn update_providers(&mut self, providers: HashMap<String, Option<ProviderModel>>) {
    let mut culled_list = HashMap::new();
    for (ns, model) in providers {
      if self.definition.providers.contains(&ns) {
        culled_list.insert(ns, model);
      }
    }
    trace!(
      "MODEL:SC[{}]:UPDATE_PROVIDERS[{}]",
      self.get_name(),
      culled_list.iter().map(|(k, _)| k).join(", ")
    );
    self.providers = culled_list;
  }

  pub(crate) fn commit_providers<T: AsRef<str>>(
    &mut self,
    providers: Vec<(T, Option<ProviderModel>)>,
  ) -> Result<()> {
    let mut map = HashMap::new();
    for (ns, model) in providers {
      let ns = ns.as_ref().to_owned();
      map.insert(ns, model);
    }
    self.update_providers(map);
    self.partial_initialization()
  }

  #[cfg(test)]
  pub(crate) fn allow_providers<T: AsRef<str>>(&mut self, namespaces: &[T]) {
    for ns in namespaces {
      self.definition.providers.push(ns.as_ref().to_owned());
    }
  }

  pub(crate) fn commit_self_provider(&mut self, provider: ProviderModel) -> Result<()> {
    trace!("MODEL:SC[{}]:UPDATE_SELF", self.get_name());
    self
      .providers
      .insert(SELF_NAMESPACE.to_owned(), Some(provider));
    self.partial_initialization()
  }

  /// Gets a [ComponentModel] by component instance string.
  pub(crate) fn get_component_model_by_instance(
    &self,
    instance: &str,
  ) -> Option<(String, ComponentModel)> {
    self
      .instances
      .get(instance)
      .and_then(|id| self.get_component_model(id))
  }

  /// Gets a [ComponentModel] by component id.
  pub(crate) fn get_component_model(&self, id: &str) -> Option<(String, ComponentModel)> {
    let (ns, name) = match parse_id(id) {
      Ok(result) => result,
      Err(_) => return None,
    };
    match self.providers.get(ns) {
      Some(Some(provider)) => provider
        .components
        .get(name)
        .cloned()
        .map(|model| (ns.to_owned(), model)),
      _ => None,
    }
  }

  /// Gets a ComponentDefinition by component instance string.
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

  pub(crate) fn get_signature(&self) -> Option<&SchematicSignature> {
    self.signature.as_ref()
  }

  pub(crate) fn get_schematic_outputs(&self) -> impl Iterator<Item = &ConnectionTargetDefinition> {
    self
      .definition
      .connections
      .iter()
      .filter(|conn| conn.to.matches_instance(SCHEMATIC_OUTPUT))
      .map(|conn| &conn.to)
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

  pub(crate) fn get_outputs(&self, instance: &str) -> Vec<ConnectionTargetDefinition> {
    match self.instances.get(instance) {
      Some(id) => match self.get_component_model(id) {
        Some((_, component)) => component
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

#[cfg(test)]
mod tests {

  #[allow(unused_imports)]
  use crate::test::prelude::{
    assert_eq,
    *,
  };
  #[test_logger::test]
  fn test_basics() -> TestResult<()> {
    let schematic_name = "logger";
    let def = load_network_definition("./src/models/test-manifests/logger.yaml")?;
    let model = SchematicModel::try_from(def.schematics[0].clone())?;
    assert_eq!(model.get_name(), schematic_name);

    Ok(())
  }

  #[test_logger::test]
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
