use crate::port::PortReference;
use crate::schematic::ConnectionIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct Connection {
  pub(crate) from: PortReference,
  pub(crate) to: PortReference,
  pub(crate) index: ConnectionIndex,
}

impl Connection {
  pub(crate) fn new(from: PortReference, to: PortReference, index: ConnectionIndex) -> Self {
    Self { from, to, index }
  }

  #[must_use]
  pub fn from(&self) -> &PortReference {
    &self.from
  }

  #[must_use]
  pub fn to(&self) -> &PortReference {
    &self.to
  }
}

impl std::fmt::Display for Connection {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}=>{}", self.index, self.from, self.to)
  }
}
