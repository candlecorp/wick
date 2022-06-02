use super::*;
use crate::component::{Component, ComponentKind};
use crate::{ComponentIndex, Schematic};
#[derive(Debug, Clone)]
#[must_use]
pub struct ComponentHop<'graph, DATA> {
  index: ComponentIndex,
  schematic: &'graph Schematic<DATA>,
}

impl<'graph, DATA> std::fmt::Display for ComponentHop<'graph, DATA>
where
  DATA: Clone,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name())
  }
}

impl<'graph, DATA> ComponentHop<'graph, DATA>
where
  DATA: Clone,
{
  pub fn new(schematic: &'graph Schematic<DATA>, index: ComponentIndex) -> Self {
    Self { schematic, index }
  }

  pub fn downstreams(&self) -> Connections<'graph, DATA> {
    let component = &self.schematic.components[self.index];
    let connections = component.all_downstreams();
    Connections::new(self.schematic, connections)
  }

  pub fn upstreams(&self) -> Connections<'graph, DATA> {
    let component = &self.schematic.components[self.index];
    let connections = component.all_upstreams();
    Connections::new(self.schematic, connections)
  }

  pub fn into_outputs(self) -> Ports<'graph, DATA> {
    let component = &self.schematic.components[self.index];
    let ports = component.outputs().iter().map(|p| p.detached()).collect();
    Ports::new(self.schematic, component.index(), ports)
  }

  pub fn into_inputs(self) -> Ports<'graph, DATA> {
    let component = &self.schematic.components[self.index];
    let ports = component.inputs().iter().map(|p| p.detached()).collect();
    Ports::new(self.schematic, component.index(), ports)
  }

  #[must_use]
  pub fn name(&self) -> &str {
    self.schematic.components[self.index].id()
  }

  #[must_use]
  pub fn index(&self) -> ComponentIndex {
    self.index
  }

  pub fn inner(&self) -> &Component<DATA> {
    &self.schematic.components[self.index]
  }

  pub fn kind(&self) -> &ComponentKind {
    self.schematic.components[self.index].kind()
  }
}
