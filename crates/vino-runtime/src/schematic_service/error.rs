use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

use crate::dev::prelude::*;

#[derive(Error, Debug)]
pub enum SchematicError {
  #[error("Provider {0} allowed by schematic but not found in network")]
  ProviderNotFound(String),
  #[error("Schematic model not initialized")]
  ModelNotInitialized,
  #[error("Transaction {0} not found")]
  TransactionNotFound(String),
  #[error("Instance {0} not found")]
  InstanceNotFound(String),
  #[error("Schematic failed pre-request condition: {0}")]
  FailedPreRequestCondition(String),
  #[error("Schematic channel closed while data still available. This can happen when the client disconnects early either due to an error or acting on the stream without waiting for it to complete.")]
  SchematicClosedEarly,
  #[error("Model invalid after validation: {0}")]
  InvalidModel(u32),
  #[error(transparent)]
  CommonError(#[from] CommonError),
  #[error(transparent)]
  ValidationError(#[from] ValidationError),
  #[error(transparent)]
  ComponentError(#[from] ProviderError),
  #[error(transparent)]
  EntityError(#[from] vino_entity::Error),
  #[error(transparent)]
  InternalError(#[from] InternalError),
  #[error(transparent)]
  TransactionChannelError(#[from] SendError<TransactionUpdate>),
  #[error(transparent)]
  ModelError(#[from] SchematicModelError),
  #[error(transparent)]
  DefaultsError(#[from] serde_json::error::Error),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error(transparent)]
  ManifestError(#[from] vino_manifest::Error),
}
