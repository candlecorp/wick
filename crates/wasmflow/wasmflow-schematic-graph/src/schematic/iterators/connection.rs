use super::*;
use crate::{Connection, ConnectionIndex};

#[derive(Debug, Clone)]
#[must_use]
pub struct Connections<'graph, DATA> {
  schematic: &'graph Schematic<DATA>,
  pub(super) connections: Vec<ConnectionIndex>,
  pub(super) cur_index: usize,
}

impl<'graph, DATA> Connections<'graph, DATA> {
  pub(crate) fn new(schematic: &'graph Schematic<DATA>, connections: Vec<ConnectionIndex>) -> Self {
    Self {
      schematic,
      connections,
      cur_index: 0,
    }
  }

  #[must_use]
  pub fn len(&self) -> usize {
    self.connections.len()
  }

  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.connections.is_empty()
  }
}

impl<'graph, DATA> Iterator for Connections<'graph, DATA>
where
  DATA: Clone,
{
  type Item = ConnectionRef<'graph, DATA>;

  fn next(&mut self) -> Option<ConnectionRef<'graph, DATA>> {
    let result = self
      .connections
      .get(self.cur_index)
      .map(|index| ConnectionRef::new(self.schematic, *index));
    self.cur_index += 1;
    result
  }
}

impl<'graph, DATA> std::fmt::Display for Connections<'graph, DATA>
where
  DATA: Clone,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for (index, _) in self.connections.iter().enumerate() {
      let comma = if index < (self.connections.len() - 1) { ", " } else { "" };
      if index == self.cur_index {
        write!(
          f,
          ">>>{}<<<{}",
          display_connection(self.schematic, get_connection(self.schematic, index)),
          comma
        )?;
      } else {
        write!(
          f,
          "{}{}",
          display_connection(self.schematic, get_connection(self.schematic, index)),
          comma
        )?;
      }
    }
    if self.cur_index >= self.connections.len() {
      write!(f, ", >>>DONE<<<")?;
    }
    Ok(())
  }
}

#[derive(Debug, Clone)]
#[must_use]
pub struct ConnectionRef<'graph, DATA> {
  schematic: &'graph Schematic<DATA>,
  pub(crate) index: ConnectionIndex,
}

impl<'graph, DATA> ConnectionRef<'graph, DATA>
where
  DATA: Clone,
{
  pub(crate) fn new(schematic: &'graph Schematic<DATA>, index: ConnectionIndex) -> Self {
    Self { schematic, index }
  }

  pub fn from(&self) -> Port<DATA> {
    let connection = &self.schematic.connections[self.index];
    Port::new(self.schematic, connection.from)
  }

  pub fn to(&self) -> Port<DATA> {
    let connection = &self.schematic.connections[self.index];
    Port::new(self.schematic, connection.to)
  }

  pub fn inner(&self) -> &Connection<DATA> {
    get_connection(self.schematic, self.index)
  }

  #[must_use]
  pub fn index(&self) -> ConnectionIndex {
    self.index
  }
}

impl<'graph, DATA> std::fmt::Display for ConnectionRef<'graph, DATA>
where
  DATA: Clone,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", display_connection(self.schematic, self.inner()))
  }
}

fn display_connection<DATA>(schematic: &Schematic<DATA>, connection: &Connection<DATA>) -> String
where
  DATA: Clone,
{
  let from_component = &schematic.components[connection.from.component_index];
  let from_port = &from_component.outputs()[connection.from.port_index];
  let to_component = &schematic.components[connection.to.component_index];
  let to_port = &to_component.inputs()[connection.to.port_index];
  format!("{}[{}]=>{}[{}]", from_component, from_port, to_component, to_port)
}
