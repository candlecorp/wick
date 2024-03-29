mod dot;
pub mod iterators;

use std::collections::HashMap;

use self::iterators::{Connections, SchematicWalker, WalkDirection};
use crate::connection::Connection;
use crate::error::Error;
use crate::node::{Node, NodeKind, NodePort};
use crate::port::PortReference;
use crate::{NodeReference, PortDirection};

pub type ConnectionIndex = usize;
pub type NodeIndex = usize;
pub type PortIndex = usize;

pub const SCHEMATIC_INPUT: &str = "<input>";
pub const SCHEMATIC_INPUT_INDEX: NodeIndex = 0;
pub const SCHEMATIC_OUTPUT: &str = "<output>";
pub const SCHEMATIC_OUTPUT_INDEX: NodeIndex = 1;

pub const NS_SCHEMATIC: &str = "__schematic__";

#[derive(Debug, Clone)]
pub struct Schematic<DATA> {
  name: String,
  input: NodeIndex,
  inherent: NodeIndex,
  output: NodeIndex,
  nodes: Vec<Node<DATA>>,
  node_map: HashMap<String, NodeIndex>,
  connections: Vec<Connection<DATA>>,
}

impl<DATA> PartialEq for Schematic<DATA> {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name && self.input == other.input && self.output == other.output
  }
}

impl<DATA> Schematic<DATA>
where
  DATA: Clone,
{
  pub fn new<T: Into<String>>(name: T, input_data: DATA, output_data: DATA) -> Self {
    let nodes = vec![
      Node::new(SCHEMATIC_INPUT, SCHEMATIC_INPUT_INDEX, NodeKind::input(), input_data),
      Node::new(
        SCHEMATIC_OUTPUT,
        SCHEMATIC_OUTPUT_INDEX,
        NodeKind::output(),
        output_data,
      ),
    ];
    let node_indices = HashMap::from([
      (SCHEMATIC_INPUT.to_owned(), SCHEMATIC_INPUT_INDEX),
      (SCHEMATIC_OUTPUT.to_owned(), SCHEMATIC_OUTPUT_INDEX),
    ]);

    Self {
      name: name.into(),
      input: 0,
      output: 1,
      inherent: 2,
      nodes,
      node_map: node_indices,
      connections: Default::default(),
    }
  }

  #[must_use]
  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn input(&self) -> &Node<DATA> {
    &self.nodes[self.input]
  }

  pub fn output(&self) -> &Node<DATA> {
    &self.nodes[self.output]
  }

  pub fn inherent(&self) -> &Node<DATA> {
    &self.nodes[self.inherent]
  }

  pub fn connections(&self) -> &[Connection<DATA>] {
    &self.connections
  }

  pub fn get_port(&self, port: &PortReference) -> &NodePort {
    match port.direction() {
      PortDirection::In => &self.nodes()[port.node_index()].inputs()[port.port_index()],
      PortDirection::Out => &self.nodes()[port.node_index()].outputs()[port.port_index()],
    }
  }

  #[must_use]
  pub fn get_ports(&self) -> Vec<PortReference> {
    self
      .nodes
      .iter()
      .flat_map(|c| {
        let mut refs = c.input_refs();
        refs.append(&mut c.output_refs());
        refs
      })
      .collect()
  }

  #[must_use]
  pub fn get_port_name(&self, port: &PortReference) -> &str {
    self.get_port(port).name()
  }

  pub fn add_input<T: Into<String>>(&mut self, port: T) -> PortReference {
    let input = self.get_mut(self.input).unwrap();
    let port = port.into();
    input.add_input(port.clone());
    input.add_output(port)
  }

  pub fn add_output<T: Into<String>>(&mut self, port: T) -> PortReference {
    let output = self.get_mut(self.output).unwrap();
    let port = port.into();
    output.add_output(port.clone());
    output.add_input(port)
  }

  pub fn nodes(&self) -> &[Node<DATA>] {
    &self.nodes
  }

  #[must_use]
  pub fn used_nodes(&self) -> Vec<&Node<DATA>> {
    let mut nodes_connected_to_input: HashMap<String, _> = SchematicWalker::new_from_input(self)
      .filter_map(|hop| match hop {
        iterators::SchematicHop::Node(n) => Some((n.name().to_owned(), n.inner())),
        _ => None,
      })
      .collect();
    let nodes_connected_to_output: HashMap<String, _> = SchematicWalker::new_from_output(self)
      .filter_map(|hop| match hop {
        iterators::SchematicHop::Node(n) => Some((n.name().to_owned(), n.inner())),
        _ => None,
      })
      .collect();
    nodes_connected_to_input.extend(nodes_connected_to_output);
    nodes_connected_to_input.into_values().collect()
  }

  #[must_use]
  pub fn get(&self, index: NodeIndex) -> Option<&Node<DATA>> {
    self.nodes.get(index)
  }

  #[must_use]
  pub fn get_mut(&mut self, index: NodeIndex) -> Option<&mut Node<DATA>> {
    self.nodes.get_mut(index)
  }

  #[must_use]
  pub fn find(&self, name: &str) -> Option<&Node<DATA>> {
    self.node_map.get(name).map(|index| &self.nodes[*index])
  }

  pub fn find_mut(&mut self, name: &str) -> Option<&mut Node<DATA>> {
    self.node_map.get_mut(name).map(|index| &mut self.nodes[*index])
  }

  #[must_use]
  pub fn get_port_connections(&self, port: &NodePort) -> Vec<&Connection<DATA>> {
    let mut connections = Vec::new();
    for i in port.connections() {
      connections.push(&self.connections[*i]);
    }
    connections
  }

  pub fn get_connections(&self) -> &[Connection<DATA>] {
    &self.connections
  }

  #[must_use]
  pub fn downstreams_from(&self, node: NodeIndex) -> Option<Vec<Connections<DATA>>> {
    self.nodes.get(node).map(|node| {
      let mut list = Vec::new();
      for port in node.outputs() {
        list.push(Connections::new(self, port.connections().to_owned()));
      }
      list
    })
  }

  #[must_use]
  pub fn downstream_connections<T: AsRef<PortReference>>(&self, port: T) -> Option<Connections<DATA>> {
    let port = port.as_ref();
    self
      .get(port.node_index)
      .and_then(|node| node.output_connections(port.port_index))
      .map(|indices| Connections::new(self, indices.clone()))
  }

  #[must_use]
  pub fn upstreams_from(&self, node: NodeIndex) -> Option<Vec<Connections<DATA>>> {
    self.nodes.get(node).map(|node| {
      let mut list = Vec::new();
      for port in node.outputs() {
        list.push(Connections::new(self, port.connections().to_owned()));
      }
      list
    })
  }

  #[must_use]
  pub fn upstream_connections<T: AsRef<PortReference>>(&self, port: T) -> Option<Connections<DATA>> {
    let port = port.as_ref();
    self
      .get(port.node_index)
      .and_then(|node| node.input_connections(port.port_index))
      .map(|indices| Connections::new(self, indices.clone()))
  }

  pub fn add_external<T: Into<String>>(&mut self, name: T, reference: NodeReference, data: DATA) -> NodeIndex {
    let name = name.into();
    self.add_node(name, NodeKind::External(reference), data)
  }

  pub fn add_and_get_mut<T: Into<String>>(&mut self, name: T, reference: NodeReference, data: DATA) -> &mut Node<DATA> {
    let index = self.add_node(name.into(), NodeKind::External(reference), data);
    self.get_mut(index).unwrap()
  }

  pub fn add_inherent<T: Into<String>>(&mut self, name: T, reference: NodeReference, data: DATA) -> NodeIndex {
    let name = name.into();
    self.add_node(name, NodeKind::Inherent(reference), data)
  }

  fn add_node(&mut self, name: String, kind: NodeKind, data: DATA) -> NodeIndex {
    let existing_index = self.node_map.get(&name);

    match existing_index {
      Some(index) => *index,
      None => {
        let index = self.nodes.len();
        let node = Node::new(&name, index, kind, data);
        self.nodes.push(node);
        self.node_map.insert(name, index);
        index
      }
    }
  }

  pub fn connect(&mut self, from: PortReference, to: PortReference, data: DATA) -> Result<(), Error> {
    let connection_index = self.connections.len();
    let downstream_node = &mut self.nodes[to.node_index];
    downstream_node.connect_input(to.port_index, connection_index)?;
    let upstream_node = &mut self.nodes[from.node_index];
    upstream_node.connect_output(from.port_index, connection_index)?;
    let connection = Connection::new(from, to, connection_index, data);
    self.connections.push(connection);
    Ok(())
  }

  pub fn walker(&self) -> SchematicWalker<DATA> {
    SchematicWalker::new_from_input(self)
  }

  pub fn walk_from_output(&self) -> SchematicWalker<DATA> {
    SchematicWalker::new_from_output(self)
  }

  pub fn walk_from_port<T: AsRef<PortReference>>(&self, port: T, direction: WalkDirection) -> SchematicWalker<DATA> {
    let mut port = *port.as_ref();
    // If we're walking from an output port, start at the complementary input port.
    // This is to make sure we walk the upstream connections, not the entire tree.
    // Use walk_from_output() to walk the entire tree upward.
    if port.node_index == self.output {
      port.direction = PortDirection::In;
    }
    SchematicWalker::from_port(self, port, direction)
  }

  #[must_use]
  pub fn render_dot(&self) -> String {
    dot::render(self)
  }
}
