use std::path::PathBuf;

use thiserror::Error;
use wick_packet::Entity;

use crate::dev::prelude::*;

fn display_path(pb: &Option<PathBuf>) -> String {
  pb.as_ref()
    .map_or_else(|| "<unknown>".into(), |p| p.to_string_lossy().to_string())
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ScopeError {
  #[error("Could not start interpreter from '{}': {1}", display_path(.0))]
  InterpreterInit(Option<PathBuf>, Box<flow_graph_interpreter::error::InterpreterError>),

  #[error("Could not generate graph from '{}': {1}", display_path(.0))]
  Graph(Option<PathBuf>, Box<flow_graph_interpreter::graph::GraphError>),

  #[error("{0} configuration referenced import '{1}' which could not be found")]
  NotFound(Context, String),

  #[error("Could not start runtime from '{}': {1}", display_path(.0))]
  RuntimeInit(Option<PathBuf>, String),

  #[error("Could not complete building the runtime. Component {0} failed to initialize: {1}")]
  ComponentInit(String, String),

  #[error("Component failed to initialize, {0}")]
  Setup(wick_packet::Error),

  #[error("Component signature mismatch. Signature reported by instantiated component at {} differs from configured signature in {}. For WebAssembly, use `wick wasm inspect` to view the embedded signature to verify its contents and update the manifest signature.", .0.to_string_lossy(), .1.to_string_lossy())]
  ComponentSignature(PathBuf, PathBuf),

  #[error(transparent)]
  FlowGraph(#[from] Box<flow_graph::error::Error>),

  #[error(transparent)]
  Manifest(#[from] Box<wick_config::Error>),

  #[error(transparent)]
  Asset(#[from] wick_config::AssetError),

  #[error("requirement {0} not fulfilled")]
  RequirementUnsatisfied(String),

  #[error(transparent)]
  NativeComponent(#[from] flow_component::ComponentError),

  #[error(transparent)]
  Wasm(#[from] Box<wick_component_wasm::Error>),

  #[error("constraint not met, {0}")]
  InvalidConstraint(ConstraintFailure),

  #[error("Internal error: {0}")]
  InternalError(InternalError),

  #[error(transparent)]
  Configuration(wick_packet::Error),
}

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ConstraintFailure {
  #[error("component {} not found", .0.component_id())]
  ComponentNotFound(Entity),
  #[error("operation {} not found in component {} (out of [{}])",.0.operation_id(),.0.component_id(), .1.join(", "))]
  OperationNotFound(Entity, Vec<String>),
  #[error("input named {1} not found in operation {0}")]
  InputNotFound(Entity, String),
  #[error("output named {1} not found in operation {0}")]
  OutputNotFound(Entity, String),
}

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum InternalError {
  MissingResolver = 1,
  InitTypeImport = 2, // tried to initialize a Type manifest as a Component
}

impl std::fmt::Display for InternalError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", *self as u16)
  }
}

impl From<ScopeError> for ComponentError {
  fn from(e: ScopeError) -> Self {
    ComponentError::ScopeError(e.to_string())
  }
}

impl From<wick_component_wasm::Error> for ScopeError {
  fn from(e: wick_component_wasm::Error) -> Self {
    ScopeError::Wasm(Box::new(e))
  }
}

impl From<flow_graph::error::Error> for ScopeError {
  fn from(e: flow_graph::error::Error) -> Self {
    ScopeError::FlowGraph(Box::new(e))
  }
}

impl From<wick_config::Error> for ScopeError {
  fn from(e: wick_config::Error) -> Self {
    ScopeError::Manifest(Box::new(e))
  }
}
