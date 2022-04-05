use std::collections::HashMap;

use crate::error::Error;
use crate::port::{PortDirection, PortReference};
use crate::schematic::{ComponentIndex, ConnectionIndex, PortIndex};
use crate::util::AsStr;

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub enum ComponentKind {
  Input(ComponentReference),
  Output(ComponentReference),
  Inherent(ComponentReference),
  External(ComponentReference),
}

impl ComponentKind {
  pub fn input() -> Self {
    ComponentKind::Input(ComponentReference {
      name: crate::SCHEMATIC_INPUT.to_owned(),
      namespace: crate::NS_SCHEMATIC.to_owned(),
    })
  }
  pub fn output() -> Self {
    ComponentKind::Output(ComponentReference {
      name: crate::SCHEMATIC_OUTPUT.to_owned(),
      namespace: crate::NS_SCHEMATIC.to_owned(),
    })
  }
  pub fn cref(&self) -> &ComponentReference {
    match self {
      ComponentKind::Input(c) => c,
      ComponentKind::Output(c) => c,
      ComponentKind::Inherent(c) => c,
      ComponentKind::External(c) => c,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct ComponentReference {
  name: String,
  namespace: String,
}

impl ComponentReference {
  pub fn new<T: AsStr, U: AsStr>(namespace: T, name: U) -> Self {
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

impl std::fmt::Display for ComponentReference {
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
        ComponentKind::Input(v) => format!("Input({})", v),
        ComponentKind::Output(v) => format!("Output({})", v),
        ComponentKind::Inherent(v) => format!("Inherent({})", v),
        ComponentKind::External(v) => format!("External({})", v),
      }
    )
  }
}

#[derive(Debug, Clone)]
#[must_use]
pub struct Component<DATA> {
  pub name: String,
  kind: ComponentKind,
  index: ComponentIndex,
  data: Option<DATA>,
  inputs: PortList,
  outputs: PortList,
}

impl<DATA> PartialEq for Component<DATA> {
  fn eq(&self, other: &Self) -> bool {
    self.index == other.index
  }
}

impl<DATA> Component<DATA>
where
  DATA: Clone,
{
  pub(crate) fn new<T: AsStr>(name: T, index: ComponentIndex, kind: ComponentKind, data: Option<DATA>) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      kind,
      index,
      data,
      inputs: PortList::new(PortDirection::In),
      outputs: PortList::new(PortDirection::Out),
    }
  }

  pub fn kind(&self) -> &ComponentKind {
    &self.kind
  }

  pub fn cref(&self) -> &ComponentReference {
    self.kind.cref()
  }

  #[must_use]
  pub fn index(&self) -> ComponentIndex {
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

  pub fn add_input<T: AsStr>(&mut self, port: T) -> PortReference {
    let port_ref = match self.kind {
      ComponentKind::Output(_) => {
        self.outputs.add(&port, self.index);
        self.inputs.add(&port, self.index)
      }
      ComponentKind::Input(_) | ComponentKind::Inherent(_) => {
        // Input/Output components have the same ports in & out.
        panic!("You can not manually add inputs to {} components", self.kind);
      }
      ComponentKind::External(_) => self.inputs.add(&port, self.index),
    };
    trace!(
      index = self.index,
      port = port.as_ref(),
      component = self.id(),
      r#ref = %port_ref,
      "added input port"
    );
    port_ref
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

  pub fn add_output<T: AsStr>(&mut self, port: T) -> PortReference {
    let port_ref = match self.kind {
      ComponentKind::Input(_) | ComponentKind::Inherent(_) => {
        self.inputs.add(&port, self.index);
        self.outputs.add(&port, self.index)
      }
      ComponentKind::Output(_) => {
        // Input/Output components have the same ports in & out.
        panic!("You can not manually add outputs to {} components", self.kind);
      }
      ComponentKind::External(_) => self.outputs.add(&port, self.index),
    };
    trace!(
      index = self.index,
      port = port.as_ref(),
      component = self.id(),
      r#ref = %port_ref,
      "added output port"
    );

    port_ref
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

  fn add<T: AsStr>(&mut self, port_name: T, component_index: ComponentIndex) -> PortReference {
    let name = port_name.as_ref();
    let existing_index = self.map.get(name);
    match existing_index {
      Some(index) => self.list[*index].port,
      None => {
        let index = self.list.len();
        let port_ref = PortReference::new(component_index, index, self.direction);
        self.map.insert(name.to_owned(), index);
        let port = ComponentPort::new(name, port_ref);
        self.list.push(port);
        port_ref
      }
    }
  }

  fn add_connection(&mut self, port: PortIndex, connection: ConnectionIndex) -> Result<(), Error> {
    let component_port = self.list.get_mut(port).ok_or(Error::InvalidPortIndex(port))?;

    if component_port.direction() == &PortDirection::In && !component_port.connections.is_empty() {
      return Err(Error::MultipleInputConnections(component_port.to_string()));
    }
    component_port.connections.push(connection);
    Ok(())
  }

  fn port_connections(&self, port: PortIndex) -> Option<&Vec<ConnectionIndex>> {
    self.list.get(port).map(|component| &component.connections)
  }

  fn all_connections(&self) -> Vec<ConnectionIndex> {
    self.list.iter().cloned().flat_map(|port| port.connections).collect()
  }
}

impl<DATA> std::fmt::Display for Component<DATA> {
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
  pub(crate) fn new<T: AsStr>(name: T, port: PortReference) -> Self {
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
  pub fn detached(&self) -> PortReference {
    self.port
  }

  pub fn direction(&self) -> &PortDirection {
    self.port.direction()
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
