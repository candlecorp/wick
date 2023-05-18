use std::collections::HashMap;

use crate::error::Error;
use crate::port::{PortDirection, PortReference};
use crate::schematic::{ConnectionIndex, NodeIndex, PortIndex};
use crate::util::AsStr;

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub enum NodeKind {
  Input(NodeReference),
  Output(NodeReference),
  Inherent(NodeReference),
  External(NodeReference),
}

impl NodeKind {
  pub fn input() -> Self {
    NodeKind::Input(NodeReference {
      name: crate::SCHEMATIC_INPUT.to_owned(),
      component_id: crate::NS_SCHEMATIC.to_owned(),
    })
  }
  pub fn output() -> Self {
    NodeKind::Output(NodeReference {
      name: crate::SCHEMATIC_OUTPUT.to_owned(),
      component_id: crate::NS_SCHEMATIC.to_owned(),
    })
  }
  pub fn cref(&self) -> &NodeReference {
    match self {
      NodeKind::Input(c) => c,
      NodeKind::Output(c) => c,
      NodeKind::Inherent(c) => c,
      NodeKind::External(c) => c,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct NodeReference {
  name: String,
  component_id: String,
}

impl NodeReference {
  pub fn new<T: AsStr, U: AsStr>(component_id: T, name: U) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      component_id: component_id.as_ref().to_owned(),
    }
  }

  #[must_use]
  pub fn component_id(&self) -> &str {
    &self.component_id
  }

  #[must_use]
  pub fn name(&self) -> &str {
    &self.name
  }
}

impl std::fmt::Display for NodeReference {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}::{}", self.component_id, self.name)
  }
}

impl std::fmt::Display for NodeKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        NodeKind::Input(v) => format!("Input({})", v),
        NodeKind::Output(v) => format!("Output({})", v),
        NodeKind::Inherent(v) => format!("Inherent({})", v),
        NodeKind::External(v) => format!("External({})", v),
      }
    )
  }
}

#[derive(Debug, Clone)]
#[must_use]
pub struct Node<DATA> {
  pub name: String,
  kind: NodeKind,
  index: NodeIndex,
  data: Option<DATA>,
  inputs: PortList,
  outputs: PortList,
}

impl<DATA> PartialEq for Node<DATA> {
  fn eq(&self, other: &Self) -> bool {
    self.index == other.index
  }
}

impl<DATA> Node<DATA>
where
  DATA: Clone,
{
  pub(crate) fn new<T: AsStr>(name: T, index: NodeIndex, kind: NodeKind, data: Option<DATA>) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      kind,
      index,
      data,
      inputs: PortList::new(PortDirection::In),
      outputs: PortList::new(PortDirection::Out),
    }
  }

  pub fn kind(&self) -> &NodeKind {
    &self.kind
  }

  pub fn cref(&self) -> &NodeReference {
    self.kind.cref()
  }

  #[must_use]
  pub fn index(&self) -> NodeIndex {
    self.index
  }

  #[must_use]
  pub fn id(&self) -> &str {
    &self.name
  }

  #[must_use]
  pub fn data(&self) -> &Option<DATA> {
    &self.data
  }

  #[must_use]
  pub fn has_data(&self) -> bool {
    self.data.is_some()
  }

  pub fn inputs(&self) -> &[NodePort] {
    self.inputs.inner()
  }

  #[must_use]
  pub fn input_refs(&self) -> Vec<PortReference> {
    self.inputs.inner().iter().map(|c| c.port).collect()
  }

  #[must_use]
  pub fn find_input(&self, name: &str) -> Option<&NodePort> {
    self.inputs.find(name)
  }

  #[must_use]
  pub fn get_input(&self, index: PortIndex) -> Option<&NodePort> {
    self.inputs.get(index)
  }

  pub fn add_input<T: AsStr>(&mut self, port: T) -> PortReference {
    match self.kind {
      NodeKind::Output(_) => {
        self.outputs.add(&port, self.index);
        self.inputs.add(&port, self.index)
      }
      NodeKind::Input(_) | NodeKind::Inherent(_) => {
        // Input/Output nodes have the same ports in & out.
        panic!("You can not manually add inputs to {} nodes", self.kind);
      }
      NodeKind::External(_) => self.inputs.add(&port, self.index),
    }
  }

  pub(crate) fn connect_input(&mut self, port: PortIndex, connection: ConnectionIndex) -> Result<(), Error> {
    self.inputs.add_connection(port, connection)
  }

  pub(crate) fn input_connections(&self, port: PortIndex) -> Option<&Vec<ConnectionIndex>> {
    self.inputs.port_connections(port)
  }

  pub(crate) fn all_upstreams(&self) -> Vec<ConnectionIndex> {
    self.inputs.all_connections()
  }

  pub fn outputs(&self) -> &[NodePort] {
    self.outputs.inner()
  }

  #[must_use]
  pub fn output_refs(&self) -> Vec<PortReference> {
    self.outputs.inner().iter().map(|c| c.port).collect()
  }

  #[must_use]
  pub fn find_output(&self, name: &str) -> Option<&NodePort> {
    self.outputs.find(name)
  }

  #[must_use]
  pub fn get_output(&self, index: PortIndex) -> Option<&NodePort> {
    self.outputs.get(index)
  }

  pub fn add_output<T: AsStr>(&mut self, port: T) -> PortReference {
    match self.kind {
      NodeKind::Input(_) | NodeKind::Inherent(_) => {
        self.inputs.add(&port, self.index);
        self.outputs.add(&port, self.index)
      }
      NodeKind::Output(_) => {
        // Input/Output nodes have the same ports in & out.
        panic!("You can not manually add outputs to {} nodes", self.kind);
      }
      NodeKind::External(_) => self.outputs.add(&port, self.index),
    }
  }

  pub(crate) fn connect_output(&mut self, port: PortIndex, connection: ConnectionIndex) -> Result<(), Error> {
    self.outputs.add_connection(port, connection)
  }

  pub(crate) fn output_connections(&self, port: PortIndex) -> Option<&Vec<ConnectionIndex>> {
    self.outputs.port_connections(port)
  }

  pub(crate) fn all_downstreams(&self) -> Vec<ConnectionIndex> {
    self.outputs.all_connections()
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PortList {
  direction: PortDirection,
  map: HashMap<String, PortIndex>,
  list: Vec<NodePort>,
}

impl PortList {
  fn new(direction: PortDirection) -> Self {
    Self {
      direction,
      map: Default::default(),
      list: Default::default(),
    }
  }
  fn find(&self, name: &str) -> Option<&NodePort> {
    self.map.get(name).map(|index| &self.list[*index])
  }

  fn get(&self, index: PortIndex) -> Option<&NodePort> {
    self.list.get(index)
  }

  fn inner(&self) -> &[NodePort] {
    &self.list
  }

  fn add<T: AsStr>(&mut self, port_name: T, node_index: NodeIndex) -> PortReference {
    let name = port_name.as_ref();
    let existing_index = self.map.get(name);
    match existing_index {
      Some(index) => self.list[*index].port,
      None => {
        let index = self.list.len();
        let port_ref = PortReference::new(node_index, index, self.direction);
        self.map.insert(name.to_owned(), index);
        let port = NodePort::new(name, port_ref);
        self.list.push(port);
        port_ref
      }
    }
  }

  fn add_connection(&mut self, port: PortIndex, connection: ConnectionIndex) -> Result<(), Error> {
    let node_port = self.list.get_mut(port).ok_or(Error::InvalidPortIndex(port))?;

    if node_port.direction() == &PortDirection::In && !node_port.connections.is_empty() {
      return Err(Error::MultipleInputConnections(node_port.to_string()));
    }
    node_port.connections.push(connection);
    Ok(())
  }

  fn port_connections(&self, port: PortIndex) -> Option<&Vec<ConnectionIndex>> {
    self.list.get(port).map(|node| &node.connections)
  }

  fn all_connections(&self) -> Vec<ConnectionIndex> {
    self.list.iter().cloned().flat_map(|port| port.connections).collect()
  }
}

impl<DATA> std::fmt::Display for Node<DATA> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name)
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct NodePort {
  name: String,
  port: PortReference,
  connections: Vec<ConnectionIndex>,
}

impl NodePort {
  pub(crate) fn new<T: AsStr>(name: T, port: PortReference) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      port,
      connections: Default::default(),
    }
  }

  #[must_use]
  pub fn is_graph_output(&self) -> bool {
    self.port.node_index == crate::schematic::SCHEMATIC_OUTPUT_INDEX
  }

  #[must_use]
  pub fn is_graph_input(&self) -> bool {
    self.port.node_index == crate::schematic::SCHEMATIC_INPUT_INDEX
  }

  #[must_use]
  pub fn connections(&self) -> &[ConnectionIndex] {
    &self.connections
  }

  #[must_use]
  pub fn name(&self) -> &str {
    &self.name
  }

  #[must_use]
  pub fn detached(&self) -> PortReference {
    self.port
  }

  pub fn direction(&self) -> &PortDirection {
    self.port.direction()
  }
}

impl From<&NodePort> for PortReference {
  fn from(port: &NodePort) -> Self {
    port.port
  }
}

impl AsRef<PortReference> for NodePort {
  fn as_ref(&self) -> &PortReference {
    &self.port
  }
}

impl std::fmt::Display for NodePort {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name)
  }
}
