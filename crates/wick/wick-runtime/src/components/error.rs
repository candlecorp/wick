use thiserror::Error;
use wick_config::config::AssetReference;

use crate::dev::prelude::*;

#[derive(Error, Debug)]
pub enum ComponentError {
  #[error("Component not found: {0}")]
  ComponentNotFound(String),

  #[error("{0}")]
  EngineError(String),

  #[error("Error initializing inner engine scope '{0}' : {1}")]
  SubScope(AssetReference, String),

  #[error("{0}")]
  Downstream(Box<dyn std::error::Error + Send + Sync>),

  #[error(transparent)]
  InvocationError(#[from] InvocationError),

  #[error(transparent)]
  Configuration(#[from] wick_config::Error),

  #[error(transparent)]
  RpcHandlerError(#[from] Box<wick_rpc::Error>),
}

impl From<wick_component_wasm::Error> for ComponentError {
  fn from(e: wick_component_wasm::Error) -> Self {
    ComponentError::Downstream(Box::new(e))
  }
}
