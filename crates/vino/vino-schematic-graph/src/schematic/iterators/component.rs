use super::*;
use crate::component::{Component, ComponentKind};
use crate::{ComponentIndex, Schematic};
#[derive(Debug, Clone)]
#[must_use]
pub struct ComponentHop<'graph> {
  schematic: &'graph Schematic,
  index: ComponentIndex,
}

impl<'graph> ComponentHop<'graph> {
  pub(crate) fn new(schematic: &'graph Schematic, index: ComponentIndex) -> Self {
    Self { schematic, index }
  }

  pub fn downstreams(&self) -> Connections<'graph> {
    let component = &self.schematic.components[self.index];
    let connections = component.all_downstreams();
    Connections::new(self.schematic, connections)
  }

  pub fn upstreams(&self) -> Connections<'graph> {
    let component = &self.schematic.components[self.index];
    let connections = component.all_upstreams();
    Connections::new(self.schematic, connections)
  }

  pub fn into_outputs(self) -> Ports<'graph> {
    let component = &self.schematic.components[self.index];
    let ports = component.outputs().iter().map(|p| p.detach()).collect();
    Ports::new(self.schematic, component.index(), ports)
  }

  pub fn into_inputs(self) -> Ports<'graph> {
    let component = &self.schematic.components[self.index];
    let ports = component.inputs().iter().map(|p| p.detach()).collect();
    Ports::new(self.schematic, component.index(), ports)
  }

  #[must_use]
  pub fn name(&self) -> &str {
    self.schematic.components[self.index].name()
  }

  pub fn inner(&self) -> &Component {
    &self.schematic.components[self.index]
  }

  pub fn kind(&self) -> &ComponentKind {
    self.schematic.components[self.index].kind()
  }
}
