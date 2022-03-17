use crate::port::PortReference;
use crate::schematic::ConnectionIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct Connection<DATA> {
  pub(crate) from: PortReference,
  pub(crate) to: PortReference,
  pub(crate) index: ConnectionIndex,
  pub(crate) data: Option<DATA>,
}

impl<DATA> Connection<DATA>
where
  DATA: Clone,
{
  pub(crate) fn new(from: PortReference, to: PortReference, index: ConnectionIndex, data: Option<DATA>) -> Self {
    Self { from, to, index, data }
  }

  #[must_use]
  pub fn index(&self) -> &ConnectionIndex {
    &self.index
  }

  #[must_use]
  pub fn from(&self) -> &PortReference {
    &self.from
  }

  #[must_use]
  pub fn to(&self) -> &PortReference {
    &self.to
  }

  #[must_use]
  pub fn data(&self) -> &Option<DATA> {
    &self.data
  }
}

impl<DATA> std::fmt::Display for Connection<DATA> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}=>{}", self.index, self.from, self.to)
  }
}
