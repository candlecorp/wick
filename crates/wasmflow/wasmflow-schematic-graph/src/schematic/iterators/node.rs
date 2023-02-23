use super::*;
use crate::node::{Node, NodeKind};
use crate::{NodeIndex, Schematic};
#[derive(Debug, Clone)]
#[must_use]
pub struct NodeHop<'graph, DATA> {
  index: NodeIndex,
  schematic: &'graph Schematic<DATA>,
}

impl<'graph, DATA> std::fmt::Display for NodeHop<'graph, DATA>
where
  DATA: Clone,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name())
  }
}

impl<'graph, DATA> NodeHop<'graph, DATA>
where
  DATA: Clone,
{
  pub fn new(schematic: &'graph Schematic<DATA>, index: NodeIndex) -> Self {
    Self { schematic, index }
  }

  pub fn downstreams(&self) -> Connections<'graph, DATA> {
    let node = &self.schematic.nodes[self.index];
    let connections = node.all_downstreams();
    Connections::new(self.schematic, connections)
  }

  pub fn upstreams(&self) -> Connections<'graph, DATA> {
    let node = &self.schematic.nodes[self.index];
    let connections = node.all_upstreams();
    Connections::new(self.schematic, connections)
  }

  pub fn into_outputs(self) -> Ports<'graph, DATA> {
    let node = &self.schematic.nodes[self.index];
    let ports = node.outputs().iter().map(|p| p.detached()).collect();
    Ports::new(self.schematic, node.index(), ports)
  }

  pub fn into_inputs(self) -> Ports<'graph, DATA> {
    let node = &self.schematic.nodes[self.index];
    let ports = node.inputs().iter().map(|p| p.detached()).collect();
    Ports::new(self.schematic, node.index(), ports)
  }

  #[must_use]
  pub fn name(&self) -> &str {
    self.schematic.nodes[self.index].id()
  }

  #[must_use]
  pub fn index(&self) -> NodeIndex {
    self.index
  }

  pub fn inner(&self) -> &Node<DATA> {
    &self.schematic.nodes[self.index]
  }

  pub fn kind(&self) -> &NodeKind {
    self.schematic.nodes[self.index].kind()
  }
}
