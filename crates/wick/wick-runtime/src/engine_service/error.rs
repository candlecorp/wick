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
  NativeComponent(#[from] Box<dyn std::error::Error + Send + Sync>),

  #[error(transparent)]
  Wasm(#[from] Box<wick_component_wasm::Error>),
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
