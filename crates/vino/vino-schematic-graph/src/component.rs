use std::collections::HashMap;

use crate::error::Error;
use crate::port::{PortDirection, PortReference};
use crate::schematic::{ComponentIndex, ConnectionIndex, PortIndex};

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub enum ComponentKind {
  Input,
  Output,
  External(ExternalReference),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalReference {
  name: String,
  namespace: String,
}

impl ExternalReference {
  pub fn new<T: AsRef<str>, U: AsRef<str>>(namespace: T, name: U) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      namespace: namespace.as_ref().to_owned(),
    }
  }

  #[must_use]
  pub fn namespace(&self) -> &str {
    &self.namespace
  }

  #[must_use]
  pub fn name(&self) -> &str {
    &self.name
  }
}

impl std::fmt::Display for ExternalReference {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}::{}", self.namespace, self.name)
  }
}

impl std::fmt::Display for ComponentKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        ComponentKind::Input => "Input".to_owned(),
        ComponentKind::Output => "Output".to_owned(),
        ComponentKind::External(v) => format!("External({})", v),
      }
    )
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct Component {
  pub name: String,
  kind: ComponentKind,
  index: ComponentIndex,
  inputs: PortList,
  outputs: PortList,
}

impl Component {
  pub(crate) fn new<T: AsRef<str>>(name: T, index: ComponentIndex, kind: ComponentKind) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      kind,
      index,
      inputs: PortList::new(PortDirection::In),
      outputs: PortList::new(PortDirection::Out),
    }
  }

  pub fn kind(&self) -> &ComponentKind {
    &self.kind
  }

  #[must_use]
  pub fn index(&self) -> ComponentIndex {
    self.index
  }

  #[must_use]
  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn inputs(&self) -> &[ComponentPort] {
    self.inputs.inner()
  }

  #[must_use]
  pub fn input_refs(&self) -> Vec<PortReference> {
    self.inputs.inner().iter().map(|c| c.port).collect()
  }

  #[must_use]
  pub fn find_input(&self, name: &str) -> Option<&ComponentPort> {
    self.inputs.find(name)
  }

  #[must_use]
  pub fn get_input(&self, index: PortIndex) -> Option<&ComponentPort> {
    self.inputs.get(index)
  }

  pub fn add_input<T: AsRef<str>>(&mut self, port: T) -> PortReference {
    match self.kind {
      ComponentKind::Output => {
        self.outputs.add(&port, self.index);
        self.inputs.add(&port, self.index)
      }
      ComponentKind::Input => {
        // Input/Output components have the same ports in & out.
        panic!("You can not manually add inputs to {} components", self.kind);
      }
      ComponentKind::External(_) => self.inputs.add(port, self.index),
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

  pub fn outputs(&self) -> &[ComponentPort] {
    self.outputs.inner()
  }

  #[must_use]
  pub fn output_refs(&self) -> Vec<PortReference> {
    self.outputs.inner().iter().map(|c| c.port).collect()
  }

  #[must_use]
  pub fn find_output(&self, name: &str) -> Option<&ComponentPort> {
    self.outputs.find(name)
  }

  #[must_use]
  pub fn get_output(&self, index: PortIndex) -> Option<&ComponentPort> {
    self.outputs.get(index)
  }

  pub fn add_output<T: AsRef<str>>(&mut self, port: T) -> PortReference {
    match self.kind {
      ComponentKind::Input => {
        self.inputs.add(&port, self.index);
        self.outputs.add(port, self.index)
      }
      ComponentKind::Output => {
        // Input/Output components have the same ports in & out.
        panic!("You can not manually add outputs to {} components", self.kind);
      }
      ComponentKind::External(_) => self.outputs.add(port, self.index),
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
  list: Vec<ComponentPort>,
}

impl PortList {
  fn new(direction: PortDirection) -> Self {
    Self {
      direction,
      map: Default::default(),
      list: Default::default(),
    }
  }
  fn find(&self, name: &str) -> Option<&ComponentPort> {
    self.map.get(name).map(|index| &self.list[*index])
  }

  fn get(&self, index: PortIndex) -> Option<&ComponentPort> {
    self.list.get(index)
  }

  fn inner(&self) -> &[ComponentPort] {
    &self.list
  }

  fn add<T: AsRef<str>>(&mut self, port_name: T, component_index: ComponentIndex) -> PortReference {
    let name = port_name.as_ref();
    let existing_index = self.map.get(name);
    match existing_index {
      Some(index) => {
        trace!("COMPONENT:PORT:GET[name={},index={}]", name, index);
        self.list[*index].port
      }
      None => {
        let index = self.list.len();
        trace!("COMPONENT:PORT:ADD[name={},index={}]", name, index);
        let port_ref = PortReference::new(component_index, index, self.direction);
        self.map.insert(name.to_owned(), index);
        let port = ComponentPort::new(name, port_ref);
        self.list.push(port);
        port_ref
      }
    }
  }

  fn add_connection(&mut self, port: PortIndex, connection: ConnectionIndex) -> Result<(), Error> {
    self
      .list
      .get_mut(port)
      .map(|component| component.connections.push(connection))
      .ok_or(Error::InvalidPortIndex(port))
  }

  fn port_connections(&self, port: PortIndex) -> Option<&Vec<ConnectionIndex>> {
    self.list.get(port).map(|component| &component.connections)
  }

  fn all_connections(&self) -> Vec<ConnectionIndex> {
    self.list.iter().cloned().flat_map(|port| port.connections).collect()
  }
}

impl std::fmt::Display for Component {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name)
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct ComponentPort {
  name: String,
  port: PortReference,
  connections: Vec<ConnectionIndex>,
}

impl ComponentPort {
  pub(crate) fn new<T: AsRef<str>>(name: T, port: PortReference) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      port,
      connections: Default::default(),
    }
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
  pub fn detach(&self) -> PortReference {
    self.port
  }
}

impl From<&ComponentPort> for PortReference {
  fn from(port: &ComponentPort) -> Self {
    port.port
  }
}

impl AsRef<PortReference> for ComponentPort {
  fn as_ref(&self) -> &PortReference {
    &self.port
  }
}

impl std::fmt::Display for ComponentPort {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name)
  }
}
