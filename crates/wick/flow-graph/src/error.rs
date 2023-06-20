use crate::PortIndex;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
  #[error("Invalid port index '{0}'")]
  InvalidPortIndex(PortIndex),
  #[error("Too many connections to input port '{0}'")]
  MultipleInputConnections(String),
  #[error("Missing downstream '{0}'")]
  MissingDownstream(String),
  #[error("Usage of inline operation '{0}' can not be discerned, it's inputs are used by multiple connections. Use the inline ID syntax to disambiguate, e.g. component::op[A] to give the operation an id of 'A'")]
  AmbiguousOperation(String),
  #[error("Invalid associated data, error was: '{0}'")]
  InvalidData(String),
}
