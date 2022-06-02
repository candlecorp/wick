use std::fmt::Display;

use crate::schematic::PortIndex;
use crate::util::AsStr;
use crate::ComponentIndex;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PortDefinition {
  pub name: String,
  pub index: PortIndex,
}

impl PortDefinition {
  pub fn new<T: AsStr>(name: T, index: PortIndex) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      index,
    }
  }
}

impl Display for PortDefinition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub struct PortReference {
  pub(crate) component_index: ComponentIndex,
  pub(crate) port_index: PortIndex,
  pub(crate) direction: PortDirection,
}

impl PortReference {
  #[must_use]
  pub fn new(component_index: ComponentIndex, port_index: PortIndex, direction: PortDirection) -> Self {
    Self {
      component_index,
      port_index,
      direction,
    }
  }

  pub fn direction(&self) -> &PortDirection {
    &self.direction
  }

  #[must_use]
  pub fn component_index(&self) -> ComponentIndex {
    self.component_index
  }

  #[must_use]
  pub fn port_index(&self) -> PortIndex {
    self.port_index
  }
}

impl AsRef<PortReference> for PortReference {
  fn as_ref(&self) -> &PortReference {
    self
  }
}

impl Display for PortReference {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.direction {
      PortDirection::In => write!(f, "{}.IN.{}", self.component_index, self.port_index),
      PortDirection::Out => write!(f, "{}.OUT.{}", self.component_index, self.port_index),
    }
  }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub enum PortDirection {
  In,
  Out,
}

impl Display for PortDirection {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        PortDirection::In => "In",
        PortDirection::Out => "Out",
      }
    )
  }
}
