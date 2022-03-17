use crate::PortIndex;

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum Error {
  #[error("Invalid port index '{0}'")]
  InvalidPortIndex(PortIndex),
}
