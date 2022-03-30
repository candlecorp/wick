use thiserror::Error;

pub use crate::network_service::error::NetworkError;
pub use crate::providers::error::ProviderError;

#[derive(Error, Debug, Clone, Copy)]
pub struct ConversionError(pub &'static str);

impl std::fmt::Display for ConversionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.0)
  }
}

#[derive(Error, Debug)]
#[error("Invocation error: {0}")]
pub struct InvocationError(pub String);

#[derive(Error, Debug)]
pub enum RuntimeError {
  #[error("Provided KeyPair has no associated seed")]
  NoSeed,
  #[error("Failed to create KeyPair from seed")]
  KeyPairFailed,

  #[error(transparent)]
  SchematicGraph(#[from] vino_schematic_graph::error::Error),
  #[error(transparent)]
  InvocationError(#[from] InvocationError),

  #[error(transparent)]
  ComponentError(#[from] ProviderError),
  #[error(transparent)]
  NetworkError(#[from] NetworkError),

  #[error("{0}")]
  Serialization(String),
  #[error("{0}")]
  InitializationFailed(String),
}
