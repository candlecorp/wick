use crate::PortIndex;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
  #[error("Invalid port index '{0}'")]
  InvalidPortIndex(PortIndex),
  #[error("Too many connections to input port '{0}'")]
  MultipleInputConnections(String),
}
