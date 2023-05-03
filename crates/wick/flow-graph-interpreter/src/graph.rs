pub mod types {
  use wick_packet::OperationConfig;

  pub(crate) static INHERENT_COMPONENT: usize = 2;
  pub(crate) type Network = flow_graph::Network<OperationConfig>;
  pub(crate) type Operation = flow_graph::Node<OperationConfig>;
  pub(crate) type OperationPort = flow_graph::NodePort;
  pub(crate) type Schematic = flow_graph::Schematic<OperationConfig>;
  pub(crate) type Port<'a> = flow_graph::iterators::Port<'a, OperationConfig>;
}

use flow_expression_parser::ast::{FlowExpression, InstanceTarget};
use flow_expression_parser::parse::CORE_ID;
use flow_graph::NodeReference;
use types::*;
use wick_config::config::{ComponentImplementation, FlowOperation};

use crate::constants::{INTERNAL_ID_INHERENT, NS_CORE, NS_INTERNAL};

#[derive(Debug)]
#[must_use]
pub(crate) struct Reference(NodeReference);

impl From<&NodeReference> for Reference {
  fn from(v: &NodeReference) -> Self {
    Self(v.clone())
  }
}

impl Reference {
  pub(crate) fn name(&self) -> &str {
    self.0.name()
  }
  pub(crate) fn namespace(&self) -> &str {
    self.0.component_id()
  }

  pub(crate) fn is_core_operation(&self, name: &str) -> bool {
    self.0.component_id() == NS_CORE && self.0.name() == name
  }
}

fn register_operation(
  mut scope: Vec<String>,
  network: &mut Network,
  flow: &FlowOperation,
) -> Result<(), flow_graph::error::Error> {
  scope.push(flow.name.clone());
  for flow in &flow.flows {
    let scope = scope.clone();
    register_operation(scope, network, flow)?;
  }
  let name = scope.join("::");
  debug!(%name, "registering operation");
  let mut schematic = Schematic::new(name);

  let index = schematic.add_inherent(CORE_ID, NodeReference::new(NS_INTERNAL, INTERNAL_ID_INHERENT), None);

  trace!(index, name = INTERNAL_ID_INHERENT, "added inherent component");

  for (name, def) in flow.instances.iter() {
    schematic.add_external(name, NodeReference::new(&def.component_id, &def.name), def.data.clone());
  }

  // inline instances
  for expression in &flow.expressions {
    match expression {
      FlowExpression::ConnectionExpression(expr) => {
        if let InstanceTarget::Path(path, id) = expr.from().instance() {
          let (component_id, op) = path.split_once("::").unwrap(); // unwrap OK if we come from a parsed config.
          schematic.add_external(id, NodeReference::new(component_id, op), None);
        }
        if let InstanceTarget::Path(path, id) = expr.to().instance() {
          let (component_id, op) = path.split_once("::").unwrap(); // unwrap OK if we come from a parsed config.
          schematic.add_external(id, NodeReference::new(component_id, op), None);
        }
      }
      FlowExpression::BlockExpression(_) => todo!(),
    }
  }

  for connection in &flow.expressions {
    match connection {
      FlowExpression::ConnectionExpression(connection) => {
        let from = connection.from();
        let to = connection.to();
        let to_port = schematic
          .find_mut(to.instance().id())
          .map(|component| component.add_input(to.port()));

        if to_port.is_none() {
          return Err(flow_graph::error::Error::MissingDownstream(
            to.instance().id().to_owned(),
          ));
        }
        let to_port = to_port.unwrap();

        if let Some(component) = schematic.find_mut(from.instance().id()) {
          let from_port = component.add_output(from.port());
          schematic.connect(from_port, to_port, None)?;
        } else {
          panic!("Can't find component {}", from.instance());
        }
      }
      FlowExpression::BlockExpression(_) => todo!(),
    }
  }
  network.add_schematic(schematic);
  Ok(())
}

pub fn from_def(manifest: &wick_config::config::ComponentConfiguration) -> Result<Network, flow_graph::error::Error> {
  let mut network = Network::new(manifest.name().clone().unwrap_or_default());

  if let ComponentImplementation::Composite(composite) = manifest.component() {
    for flow in composite.operations().values() {
      register_operation(vec![], &mut network, flow)?;
    }
  }

  Ok(network)
}
