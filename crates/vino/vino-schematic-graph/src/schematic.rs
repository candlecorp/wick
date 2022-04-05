mod dot;
pub mod iterators;

use std::collections::HashMap;

use self::iterators::{Connections, SchematicWalker, WalkDirection};
use crate::component::{Component, ComponentKind, ComponentPort};
use crate::connection::Connection;
use crate::error::Error;
use crate::port::PortReference;
use crate::util::AsStr;
use crate::{ComponentReference, PortDirection};

pub type ConnectionIndex = usize;
pub type ComponentIndex = usize;
pub type PortIndex = usize;

pub static SCHEMATIC_INPUT: &str = "<input>";
pub static SCHEMATIC_OUTPUT: &str = "<output>";

pub static NS_SCHEMATIC: &str = "__schematic__";

#[derive(Debug, Clone)]
pub struct Schematic<DATA> {
  name: String,
  input: ComponentIndex,
  inherent: ComponentIndex,
  output: ComponentIndex,
  components: Vec<Component<DATA>>,
  component_map: HashMap<String, ComponentIndex>,
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
  pub fn new<T: AsStr>(name: T) -> Self {
    let components = vec![
      Component::new(SCHEMATIC_INPUT, 0, ComponentKind::input(), None),
      Component::new(SCHEMATIC_OUTPUT, 1, ComponentKind::output(), None),
    ];
    let component_indices = HashMap::from([(SCHEMATIC_INPUT.to_owned(), 0), (SCHEMATIC_OUTPUT.to_owned(), 1)]);

    Self {
      name: name.as_ref().to_owned(),
      input: 0,
      output: 1,
      inherent: 2,
      components,
      component_map: component_indices,
      connections: Default::default(),
    }
  }

  #[must_use]
  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn input(&self) -> &Component<DATA> {
    &self.components[self.input]
  }

  pub fn output(&self) -> &Component<DATA> {
    &self.components[self.output]
  }

  pub fn inherent(&self) -> &Component<DATA> {
    &self.components[self.inherent]
  }

  pub fn connections(&self) -> &[Connection<DATA>] {
    &self.connections
  }

  pub fn get_port(&self, port: &PortReference) -> &ComponentPort {
    match port.direction() {
      PortDirection::In => &self.components()[port.component_index()].inputs()[port.port_index()],
      PortDirection::Out => &self.components()[port.component_index()].outputs()[port.port_index()],
    }
  }

  #[must_use]
  pub fn get_ports(&self) -> Vec<PortReference> {
    self
      .components
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

  pub fn add_input<T: AsStr>(&mut self, port: T) -> PortReference {
    trace!(?port, "add schematic input");
    let input = self.get_mut(self.input).unwrap();
    input.add_input(&port);
    input.add_output(port)
  }

  pub fn add_output<T: AsStr>(&mut self, port: T) -> PortReference {
    trace!(?port, "add schematic output");
    let output = self.get_mut(self.output).unwrap();
    output.add_output(&port);
    output.add_input(port)
  }

  pub fn components(&self) -> &[Component<DATA>] {
    &self.components
  }

  #[must_use]
  pub fn get(&self, index: ComponentIndex) -> Option<&Component<DATA>> {
    self.components.get(index)
  }

  pub(crate) fn get_mut(&mut self, index: ComponentIndex) -> Option<&mut Component<DATA>> {
    self.components.get_mut(index)
  }

  #[must_use]
  pub fn find<T: AsStr>(&self, name: T) -> Option<&Component<DATA>> {
    self
      .component_map
      .get(name.as_ref())
      .map(|index| &self.components[*index])
  }

  pub fn find_mut<T: AsStr>(&mut self, name: T) -> Option<&mut Component<DATA>> {
    self
      .component_map
      .get_mut(name.as_ref())
      .map(|index| &mut self.components[*index])
  }

  #[must_use]
  pub fn get_port_connections(&self, port: &ComponentPort) -> Vec<&Connection<DATA>> {
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
  pub fn downstreams_from(&self, component: ComponentIndex) -> Option<Vec<Connections<DATA>>> {
    self.components.get(component).map(|component| {
      let mut list = Vec::new();
      for port in component.outputs() {
        list.push(Connections::new(self, port.connections().to_owned()));
      }
      list
    })
  }

  #[must_use]
  pub fn downstream_connections<T: AsRef<PortReference>>(&self, port: T) -> Option<Connections<DATA>> {
    let port = port.as_ref();
    self
      .get(port.component_index)
      .and_then(|component| component.output_connections(port.port_index))
      .map(|indices| Connections::new(self, indices.clone()))
  }

  #[must_use]
  pub fn upstreams_from(&self, component: ComponentIndex) -> Option<Vec<Connections<DATA>>> {
    self.components.get(component).map(|component| {
      let mut list = Vec::new();
      for port in component.outputs() {
        list.push(Connections::new(self, port.connections().to_owned()));
      }
      list
    })
  }

  #[must_use]
  pub fn upstream_connections<T: AsRef<PortReference>>(&self, port: T) -> Option<Connections<DATA>> {
    let port = port.as_ref();
    self
      .get(port.component_index)
      .and_then(|component| component.input_connections(port.port_index))
      .map(|indices| Connections::new(self, indices.clone()))
  }

  pub fn add_external<T: AsStr>(
    &mut self,
    name: T,
    reference: ComponentReference,
    data: Option<DATA>,
  ) -> ComponentIndex {
    self.add_component(name.as_ref().to_owned(), ComponentKind::External(reference), data)
  }

  pub fn add_inherent<T: AsStr>(
    &mut self,
    name: T,
    reference: ComponentReference,
    data: Option<DATA>,
  ) -> ComponentIndex {
    self.add_component(name.as_ref().to_owned(), ComponentKind::Inherent(reference), data)
  }

  fn add_component(&mut self, name: String, kind: ComponentKind, data: Option<DATA>) -> ComponentIndex {
    let existing_index = self.component_map.get(&name);

    match existing_index {
      Some(index) => {
        trace!(name = name.as_str(), index, ?kind, "retrieving existing component");
        *index
      }
      None => {
        let index = self.components.len();
        trace!(name = name.as_str(), index, ?kind, "added component");
        let component = Component::new(&name, index, kind, data);
        self.components.push(component);
        self.component_map.insert(name, index);
        index
      }
    }
  }

  pub fn connect(&mut self, from: PortReference, to: PortReference, data: Option<DATA>) -> Result<(), Error> {
    trace!(?from, ?to, "connecting");
    let connection_index = self.connections.len();
    let downstream_component = &mut self.components[to.component_index];
    downstream_component.connect_input(to.port_index, connection_index)?;
    let upstream_component = &mut self.components[from.component_index];
    upstream_component.connect_output(from.port_index, connection_index)?;
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
    if port.component_index == self.output {
      port.direction = PortDirection::In;
    }
    SchematicWalker::from_port(self, port, direction)
  }

  #[must_use]
  pub fn render_dot(&self) -> String {
    dot::render(self)
  }
}
