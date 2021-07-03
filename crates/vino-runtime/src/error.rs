use std::path::PathBuf;
use std::sync::PoisonError;

use itertools::join;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use vino_rpc::PortSignature;

use crate::dev::prelude::*;
use crate::schematic_service::messages::PayloadReceived;

pub(crate) type BoxedErrorSyncSend = Box<dyn std::error::Error + Sync + Send>;

#[derive(Error, Debug, PartialEq)]
pub enum ValidationError {
  #[error("Schematic '{0}' has errors: {}", join(.1, ", "))]
  PostInitError(String, Vec<ValidationError>),
  #[error("Schematic '{0}' has errors: {}", join(.1, ", "))]
  EarlyError(String, Vec<ValidationError>),
  #[error("Schematic has no outputs")]
  NoOutputs,
  #[error("Schematic has no inputs")]
  NoInputs,
  #[error(transparent)]
  ModelError(#[from] SchematicModelError),
  #[error("The following component(s) have incomplete internal model(s): '{}'", join(.0, ", "))]
  MissingComponentModels(Vec<String>),
  #[error("Dangling reference(s): '{}'", join(.0, ", "))]
  DanglingReference(Vec<String>),
  #[error("Component definition(s) '{}' not fully qualified", join(.0, ", "))]
  NotFullyQualified(Vec<String>),
  #[error("Invalid output port '{}' on {}. Valid output ports are [{}]", .0.name, .1, join(.2, ", "))]
  InvalidOutputPort(PortReference, Connection, Vec<PortSignature>),
  #[error("Invalid input port '{}' on {}. Valid input ports are [{}]", .0.name, .1, join(.2, ", "))]
  InvalidInputPort(PortReference, Connection, Vec<PortSignature>),
  #[error("Invalid connections: {}", join(.0, ", "))]
  InvalidConnections(Vec<ValidationError>),
}

#[derive(Error, Debug, PartialEq)]
pub enum SchematicModelError {
  #[error("Schematic model not able to finish initialization")]
  IncompleteInitialization,
  #[error("Schematic model not initialized")]
  ModelNotInitialized,
  #[error("The reference '{0}' has an incomplete component model. Component may have failed to load or be in a partial state.")]
  MissingComponentModel(String),
}

#[derive(Error, Debug)]
pub enum SchematicError {
  #[error("Schematic model not initialized")]
  ModelNotInitialized,
  #[error("Transaction {0} not found")]
  TransactionNotFound(String),
  #[error("Reference {0} not found")]
  ReferenceNotFound(String),
  #[error("Schematic failed pre-request condition")]
  FailedPreRequestCondition(String),
  #[error("Schematic channel closed while data still available. This can happen when acting on output before waiting for the system to receive the final close and may not be a problem. Error: {0}")]
  SchematicClosedEarly(String),
  #[error(transparent)]
  CommonError(#[from] CommonError),
  #[error(transparent)]
  ValidationError(#[from] ValidationError),
  #[error(transparent)]
  KeyPairError(#[from] nkeys::error::Error),
  #[error(transparent)]
  ComponentError(#[from] ComponentError),
  #[error(transparent)]
  EntityError(#[from] vino_entity::Error),
  #[error(transparent)]
  InternalError(#[from] InternalError),
  #[error(transparent)]
  TransactionChannelError(#[from] SendError<PayloadReceived>),
  #[error(transparent)]
  ModelError(#[from] SchematicModelError),
}

#[derive(Error, Debug)]
pub enum NetworkError {
  #[error("Network not started")]
  NotStarted,
  #[error("Schematic {0} not found")]
  SchematicNotFound(String),
  #[error("Error initializing: {0}")]
  InitializationError(String),
  #[error("Maximum number of tries reached when resolving internal schematic references")]
  MaxTriesReached,
  #[error(transparent)]
  SchematicError(#[from] SchematicError),
  #[error(transparent)]
  ComponentError(#[from] ComponentError),
  #[error(transparent)]
  InternalError(#[from] InternalError),
  #[error(transparent)]
  CommonError(#[from] CommonError),
  #[error("Error executing request: {0}")]
  ExecutionError(String),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
}

#[derive(Error, Debug)]
pub enum ComponentError {
  #[error("Could not extract claims from component")]
  ClaimsError,
  #[error(transparent)]
  WascapError(#[from] wascap::Error),
  #[error("Failed to create a raw WebAssembly host")]
  WapcError,
  #[error(transparent)]
  ConversionError(#[from] ConversionError),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
  #[error("Component not found, looked in {0}")]
  NotFound(String),
  #[error(transparent)]
  OciError(#[from] OciError),
  #[error(transparent)]
  ActixMailboxError(#[from] MailboxError),
  #[error(transparent)]
  RpcError(#[from] vino_rpc::Error),
  #[error(transparent)]
  OtherUpstream(#[from] BoxedErrorSyncSend),
  #[error(transparent)]
  RpcUpstreamError(#[from] tonic::Status),
  #[error(transparent)]
  OutputError(#[from] vino_component::Error),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error("Grpc Provider error: {0}")]
  GrpcUrlProviderError(String),
  #[error(transparent)]
  InternalError(#[from] InternalError),
  #[error(transparent)]
  CommonError(#[from] CommonError),
}

#[derive(Error, Debug)]
pub enum OciError {
  #[error("Configuration disallows fetching artifacts with the :latest tag ({0})")]
  LatestDisallowed(String),
  #[error("Could not fetch '{0}': {1}")]
  OciFetchFailure(String, String),
  #[error("Could not parse OCI URL {0}: {1}")]
  OCIParseError(String, String),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
}

#[derive(Error, Debug, Clone, Copy)]
pub struct ConversionError(pub &'static str);

impl std::fmt::Display for ConversionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.0)
  }
}

#[derive(Error, Debug, Clone, Copy)]
pub struct InternalError(pub i32);

impl std::fmt::Display for InternalError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("Internal Error: {}", self.0))
  }
}

#[derive(Error, Debug)]
pub enum CommonError {
  #[error("Failed to acquire a lock: {0}")]
  LockError(String),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
  #[error("File not found {}", .0.to_string_lossy())]
  FileNotFound(PathBuf),
}

#[derive(Error, Debug)]
pub enum TransactionError {
  #[error(transparent)]
  CommonError(#[from] CommonError),
  #[error(transparent)]
  InternalError(#[from] InternalError),
  #[error("Upstream port {0} not found")]
  UpstreamNotFound(PortReference),
}

#[derive(Error, Debug)]
pub enum VinoError {
  #[error("Invocation error: {0}")]
  InvocationError(String),
  #[error(transparent)]
  CommonError(#[from] CommonError),
  #[error(transparent)]
  TransactionError(#[from] TransactionError),
  #[error("Conversion error {0}")]
  ConversionError(&'static str),
  #[error("URL parse error {0}")]
  ParseError(String),
  #[error(transparent)]
  ComponentError(#[from] ComponentError),
  #[error(transparent)]
  NetworkError(#[from] NetworkError),

  #[error("Dispatch error: {0}")]
  DispatchError(String),
  #[error("Provider error {0}")]
  ProviderError(String),
  #[error("WaPC WebAssembly Component error: {0}")]
  WapcError(String),
  #[error("Job error: {0}")]
  JobError(String),
  #[error("invalid configuration")]
  ConfigurationError,
  #[error("Could not start host: {0}")]
  HostStartFailure(String),
  #[error("Failed to deserialize configuration {0}")]
  ConfigurationDeserialization(String),
  #[error("Failed to serialize payload {0}")]
  SerializationError(rmp_serde::encode::Error),
  #[error("Failed to deserialize payload {0}")]
  DeserializationError(rmp_serde::decode::Error),

  #[error(transparent)]
  OciError(#[from] OciError),
  #[error(transparent)]
  SchematicError(#[from] SchematicError),

  #[error(transparent)]
  TonicError(#[from] tonic::transport::Error),
  #[error(transparent)]
  RpcUpstreamError(#[from] tonic::Status),
  #[error(transparent)]
  EntityError(#[from] vino_entity::Error),
  #[error(transparent)]
  RpcError(#[from] vino_rpc::Error),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error(transparent)]
  ManifestError(#[from] vino_manifest::Error),
  #[error(transparent)]
  TransportError(#[from] vino_transport::Error),
  #[error(transparent)]
  OutputError(#[from] vino_component::Error),
  #[error(transparent)]
  ActixMailboxError(#[from] MailboxError),
  #[error(transparent)]
  KeyPairError(#[from] nkeys::error::Error),

  #[error(transparent)]
  OtherUpstream(#[from] BoxedErrorSyncSend),
  #[error("General error : {0}")]
  Other(String),

  #[error(transparent)]
  IOError(#[from] std::io::Error),
}

impl<T> From<PoisonError<std::sync::MutexGuard<'_, T>>> for VinoError {
  fn from(lock_error: PoisonError<std::sync::MutexGuard<'_, T>>) -> Self {
    CommonError::LockError(lock_error.to_string()).into()
  }
}

impl<T> From<PoisonError<std::sync::MutexGuard<'_, T>>> for NetworkError {
  fn from(lock_error: PoisonError<std::sync::MutexGuard<'_, T>>) -> Self {
    CommonError::LockError(lock_error.to_string()).into()
  }
}

impl<T> From<PoisonError<std::sync::MutexGuard<'_, T>>> for SchematicError {
  fn from(lock_error: PoisonError<std::sync::MutexGuard<'_, T>>) -> Self {
    CommonError::LockError(lock_error.to_string()).into()
  }
}

impl<T> From<PoisonError<std::sync::MutexGuard<'_, T>>> for TransactionError {
  fn from(lock_error: PoisonError<std::sync::MutexGuard<'_, T>>) -> Self {
    CommonError::LockError(lock_error.to_string()).into()
  }
}

impl From<&'static str> for VinoError {
  fn from(e: &'static str) -> Self {
    VinoError::Other(e.to_owned())
  }
}
