use thiserror::Error;

use crate::dev::prelude::*;
#[derive(Error, Debug)]
pub enum NetworkError {
  #[error("Could not start interpreter from '{}': {1}", .0.as_ref().map_or_else(|| "<unknown>".into(), |p| p.to_string()))]
  InterpreterInit(Option<url::Url>, Box<flow_graph_interpreter::error::InterpreterError>),

  #[error(transparent)]
  FlowGraph(#[from] Box<flow_graph::error::Error>),

  #[error(transparent)]
  Manifest(#[from] Box<wick_config::Error>),

  #[error(transparent)]
  NativeComponent(#[from] Box<dyn std::error::Error + Send + Sync>),

  #[error(transparent)]
  Wasm(#[from] Box<wick_component_wasm::Error>),
}

impl From<NetworkError> for ComponentError {
  fn from(e: NetworkError) -> Self {
    ComponentError::NetworkError(e.to_string())
  }
}

impl From<wick_component_wasm::Error> for NetworkError {
  fn from(e: wick_component_wasm::Error) -> Self {
    NetworkError::Wasm(Box::new(e))
  }
}

impl From<flow_graph::error::Error> for NetworkError {
  fn from(e: flow_graph::error::Error) -> Self {
    NetworkError::FlowGraph(Box::new(e))
  }
}

impl From<wick_config::Error> for NetworkError {
  fn from(e: wick_config::Error) -> Self {
    NetworkError::Manifest(Box::new(e))
  }
}
