use std::collections::HashMap;

use vino_schematic_graph::{Component, ComponentKind, PortReference, Schematic};
use vino_transport::{TransportMap, TransportWrapper};

use super::error::ExecutionError;
use super::port::{InputPorts, OutputPorts, PortList, Ports};
use crate::interpreter::provider::internal::INTERNAL_PROVIDER;
type Result<T> = std::result::Result<T, ExecutionError>;

#[derive(Debug)]
#[must_use]
pub(crate) struct ComponentHandler {
  namespace: String,
  name: String,
  inputs: InputPorts,
  outputs: OutputPorts,
}

impl ComponentHandler {
  pub(super) fn new(schematic: &Schematic, component: &Component) -> Self {
    let inputs: Vec<PortReference> = component.inputs().iter().map(|p| p.detach()).collect();
    let outputs: Vec<PortReference> = component.outputs().iter().map(|p| p.detach()).collect();
    let namespace = match component.kind() {
      ComponentKind::Input => INTERNAL_PROVIDER,
      ComponentKind::Output => INTERNAL_PROVIDER,
      ComponentKind::External(comp) => comp.namespace(),
    };

    Self {
      name: component.name().to_owned(),
      namespace: namespace.to_owned(),
      inputs: InputPorts::new(PortList::new(schematic, &inputs)),
      outputs: OutputPorts::new(PortList::new(schematic, &outputs)),
    }
  }

  pub(crate) fn namespace(&self) -> &str {
    &self.namespace
  }

  pub(crate) fn name(&self) -> &str {
    &self.name
  }

  pub(crate) async fn collect_transport_map(&self) -> Result<TransportMap> {
    self.inputs.shift().await
  }

  pub(crate) fn validate_payload(&self, payload: &TransportMap) -> Result<()> {
    for (_, input) in self.inputs.iter() {
      if !payload.has(input.name()) {
        return Err(ExecutionError::MissingInput(input.name().to_owned()));
      }
    }

    Ok(())
  }

  pub(crate) fn is_ready(&self) -> bool {
    self.inputs.is_ready()
  }

  pub(crate) async fn receive(&self, port: &PortReference, value: TransportWrapper) -> Result<()> {
    self.inputs.receive(port, value).await
  }

  pub(crate) fn find_input(&self, name: &str) -> Option<&PortReference> {
    self.inputs.find(name)
  }

  pub(crate) fn output_refmap(&self) -> HashMap<String, PortReference> {
    self
      .outputs
      .iter()
      .map(|(port_ref, handler)| (handler.name().to_owned(), *port_ref))
      .collect()
  }
}
