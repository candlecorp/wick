mod dot;
pub mod iterators;

use std::collections::HashMap;

use self::iterators::{Connections, SchematicWalker, WalkDirection};
use crate::component::{Component, ComponentKind, ComponentPort};
use crate::connection::Connection;
use crate::error::Error;
use crate::port::PortReference;
use crate::{ExternalReference, PortDirection};

pub type ConnectionIndex = usize;
pub type ComponentIndex = usize;
pub type PortIndex = usize;

pub static SCHEMATIC_INPUT: &str = "<input>";
pub static SCHEMATIC_OUTPUT: &str = "<output>";

#[derive(Debug, Clone)]
pub struct Schematic {
  name: String,
  input: ComponentIndex,
  output: ComponentIndex,
  components: Vec<Component>,
  component_map: HashMap<String, ComponentIndex>,
  connections: Vec<Connection>,
}

impl PartialEq for Schematic {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name && self.input == other.input && self.output == other.output
  }
}

impl Schematic {
  pub fn new<T: AsRef<str>>(name: T) -> Self {
    let components = vec![
      Component::new(SCHEMATIC_INPUT, 0, ComponentKind::Input),
      Component::new(SCHEMATIC_OUTPUT, 1, ComponentKind::Output),
    ];
    let component_indices = HashMap::from([(SCHEMATIC_INPUT.to_owned(), 0), (SCHEMATIC_OUTPUT.to_owned(), 1)]);

    Self {
      name: name.as_ref().to_owned(),
      input: 0,
      output: 1,
      components,
      component_map: component_indices,
      connections: Default::default(),
    }
  }

  #[must_use]
  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn input(&self) -> &Component {
    &self.components[self.input]
  }

  pub fn output(&self) -> &Component {
    &self.components[self.output]
  }

  pub fn connections(&self) -> &[Connection] {
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

  pub fn add_input<T: AsRef<str>>(&mut self, port: T) -> PortReference {
    let input = self.get_mut(self.input).unwrap();
    input.add_input(port.as_ref());
    input.add_output(port)
  }

  pub fn add_output<T: AsRef<str>>(&mut self, port: T) -> PortReference {
    let output = self.get_mut(self.output).unwrap();
    output.add_output(port.as_ref());
    output.add_input(port)
  }

  pub fn components(&self) -> &[Component] {
    &self.components
  }

  #[must_use]
  pub fn get(&self, index: ComponentIndex) -> Option<&Component> {
    self.components.get(index)
  }

  pub(crate) fn get_mut(&mut self, index: ComponentIndex) -> Option<&mut Component> {
    self.components.get_mut(index)
  }

  #[must_use]
  pub fn find<T: AsRef<str>>(&self, name: T) -> Option<&Component> {
    self
      .component_map
      .get(name.as_ref())
      .map(|index| &self.components[*index])
  }

  pub fn find_mut<T: AsRef<str>>(&mut self, name: T) -> Option<&mut Component> {
    self
      .component_map
      .get_mut(name.as_ref())
      .map(|index| &mut self.components[*index])
  }

  #[must_use]
  pub fn get_connections(&self, port: &ComponentPort) -> Vec<&Connection> {
    let mut connections = Vec::new();
    for i in port.connections() {
      connections.push(&self.connections[*i]);
    }
    connections
  }

  #[must_use]
  pub fn downstreams_from(&self, component: ComponentIndex) -> Option<Vec<Connections>> {
    self.components.get(component).map(|component| {
      let mut list = Vec::new();
      for port in component.outputs() {
        list.push(Connections::new(self, port.connections().to_owned()));
      }
      list
    })
  }

  #[must_use]
  pub fn downstream_connections<T: AsRef<PortReference>>(&self, port: T) -> Option<Connections> {
    let port = port.as_ref();
    self
      .get(port.component_index)
      .and_then(|component| component.output_connections(port.port_index))
      .map(|indices| Connections::new(self, indices.clone()))
  }

  #[must_use]
  pub fn upstreams_from(&self, component: ComponentIndex) -> Option<Vec<Connections>> {
    self.components.get(component).map(|component| {
      let mut list = Vec::new();
      for port in component.outputs() {
        list.push(Connections::new(self, port.connections().to_owned()));
      }
      list
    })
  }

  #[must_use]
  pub fn upstream_connections<T: AsRef<PortReference>>(&self, port: T) -> Option<Connections> {
    let port = port.as_ref();
    self
      .get(port.component_index)
      .and_then(|component| component.input_connections(port.port_index))
      .map(|indices| Connections::new(self, indices.clone()))
  }

  pub fn add_or_get_instance<T: AsRef<str>>(&mut self, name: T, reference: ExternalReference) -> ComponentIndex {
    let existing_index = self.component_map.get(name.as_ref());

    match existing_index {
      Some(index) => {
        trace!("COMPONENT:GET[name={},index={}]", name.as_ref(), index);
        *index
      }
      None => {
        let index = self.components.len();
        trace!("COMPONENT:ADD[name={},index={}]", name.as_ref(), index);
        let component = Component::new(&name, index, ComponentKind::External(reference));
        self.components.push(component);
        self.component_map.insert(name.as_ref().to_owned(), index);
        index
      }
    }
  }

  pub fn connect(&mut self, from: PortReference, to: PortReference) -> Result<(), Error> {
    trace!("SCHEMATIC:CONNECT[{}=>{}]", from, to);
    let connection_index = self.connections.len();
    let upstream_component = &mut self.components[from.component_index];
    upstream_component.connect_output(from.port_index, connection_index)?;
    let downstream_component = &mut self.components[to.component_index];
    downstream_component.connect_input(to.port_index, connection_index)?;
    let connection = Connection::new(from, to, connection_index);
    self.connections.push(connection);
    Ok(())
  }

  pub fn walker(&self) -> SchematicWalker {
    SchematicWalker::new(self)
  }

  pub fn walk_from_port(&self, port: PortReference, direction: WalkDirection) -> SchematicWalker {
    SchematicWalker::from_port(self, port, direction)
  }

  #[must_use]
  pub fn render_dot(&self) -> String {
    dot::render(self)
  }
}
