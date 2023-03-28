pub mod types {
  pub(crate) static INHERENT_COMPONENT: usize = 2;
  pub(crate) type Network = flow_graph::Network<serde_json::Value>;
  pub(crate) type Operation = flow_graph::Node<serde_json::Value>;
  pub(crate) type OperationPort = flow_graph::NodePort;
  pub(crate) type Schematic = flow_graph::Schematic<serde_json::Value>;
  pub(crate) type Port<'a> = flow_graph::iterators::Port<'a, serde_json::Value>;
}

use flow_expression_parser::parse::CORE_ID;
use flow_graph::NodeReference;
use types::*;
use wick_config::config::ComponentImplementation;

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

  // pub(crate) fn is_static(&self) -> bool {
  //   self.0.namespace() == NS_COLLECTIONS
  // }
}

#[instrument(name = "graph", skip_all, level = "trace", fields(name=manifest.name()))]
pub fn from_def(manifest: &wick_config::config::ComponentConfiguration) -> Result<Network, flow_graph::error::Error> {
  let mut network = Network::new(manifest.name().clone().unwrap_or_default());

  if let ComponentImplementation::Composite(composite) = manifest.component() {
    for (name, flow) in composite.operations() {
      let mut schematic = Schematic::new(name.clone());

      let index = schematic.add_inherent(CORE_ID, NodeReference::new(NS_INTERNAL, INTERNAL_ID_INHERENT), None);

      trace!(index, name = INTERNAL_ID_INHERENT, "added inherent component");

      for (name, def) in flow.instances.iter() {
        schematic.add_external(name, NodeReference::new(&def.component_id, &def.name), def.data.clone());
      }

      for connection in &flow.connections {
        let from = &connection.from;
        let to = &connection.to;
        let to_port = schematic
          .find_mut(to.get_instance())
          .map(|component| component.add_input(to.get_port()));

        assert!(
          to_port.is_some(),
          "Could not find downstream instance '{}'",
          to.get_instance(),
        );
        let to_port = to_port.unwrap();

        if let Some(component) = schematic.find_mut(from.get_instance()) {
          let from_port = component.add_output(from.get_port());
          schematic.connect(from_port, to_port, None)?;
        } else {
          panic!("Can't find component {}", from.get_instance());
        }
      }
      network.add_schematic(schematic);
    }
  }

  Ok(network)
}
