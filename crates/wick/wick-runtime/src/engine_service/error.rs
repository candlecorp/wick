use thiserror::Error;

use crate::dev::prelude::*;
#[derive(Error, Debug)]
pub enum EngineError {
  #[error("Could not start interpreter from '{}': {1}", .0.as_ref().map_or_else(|| "<unknown>".into(), |p| p.clone()))]
  InterpreterInit(Option<String>, Box<flow_graph_interpreter::error::InterpreterError>),

  #[error(transparent)]
  FlowGraph(#[from] Box<flow_graph::error::Error>),

  #[error(transparent)]
  Manifest(#[from] Box<wick_config::Error>),

  #[error(transparent)]
  Asset(#[from] wick_config::AssetError),

  #[error(transparent)]
  NativeComponent(#[from] flow_component::ComponentError),

  #[error(transparent)]
  Wasm(#[from] Box<wick_component_wasm::Error>),

  #[error("Internal error: {0}")]
  InternalError(InternalError),
}

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum InternalError {
  MissingResolver = 1,
}

impl std::fmt::Display for InternalError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", *self as u16)
  }
}

impl From<EngineError> for ComponentError {
  fn from(e: EngineError) -> Self {
    ComponentError::EngineError(e.to_string())
  }
}

impl From<wick_component_wasm::Error> for EngineError {
  fn from(e: wick_component_wasm::Error) -> Self {
    EngineError::Wasm(Box::new(e))
  }
}

impl From<flow_graph::error::Error> for EngineError {
  fn from(e: flow_graph::error::Error) -> Self {
    EngineError::FlowGraph(Box::new(e))
  }
}

impl From<wick_config::Error> for EngineError {
  fn from(e: wick_config::Error) -> Self {
    EngineError::Manifest(Box::new(e))
  }
}
