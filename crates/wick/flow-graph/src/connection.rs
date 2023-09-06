use crate::port::PortReference;
use crate::schematic::ConnectionIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct Connection<DATA> {
  pub(crate) from: PortReference,
  pub(crate) to: PortReference,
  pub(crate) index: ConnectionIndex,
  pub(crate) data: DATA,
}

impl<DATA> Connection<DATA>
where
  DATA: Clone,
{
  pub(crate) const fn new(from: PortReference, to: PortReference, index: ConnectionIndex, data: DATA) -> Self {
    Self { from, to, index, data }
  }

  #[must_use]
  pub const fn index(&self) -> &ConnectionIndex {
    &self.index
  }

  #[must_use]
  pub const fn from(&self) -> &PortReference {
    &self.from
  }

  #[must_use]
  pub const fn to(&self) -> &PortReference {
    &self.to
  }

  #[must_use]
  pub const fn data(&self) -> &DATA {
    &self.data
  }
}

impl<DATA> std::fmt::Display for Connection<DATA> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}=>{}", self.index, self.from, self.to)
  }
}
