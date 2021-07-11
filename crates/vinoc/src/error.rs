use thiserror::Error;
use vino_codec::error::CodecError;

// type BoxedSyncSendError = Box<dyn std::error::Error + Sync + std::marker::Send>;

#[derive(Error, Debug)]
pub enum ControlError {
  #[error("invalid configuration")]
  ConfigurationError,
  #[error("File not found {0}")]
  FileNotFound(String),
  #[error("Configuration disallows fetching artifacts with the :latest tag ({0})")]
  LatestDisallowed(String),
  #[error("Could not start host: {0}")]
  HostStartFailure(String),
  #[error("Keypair error: {0}")]
  KeyPairError(String),
  #[error("Keypair path or string not supplied. Ensure provided keypair is valid")]
  KeyPairNotProvided,
  #[error("Failed to deserialize configuration {0}")]
  ConfigurationDeserialization(String),
  #[error(transparent)]
  LoggerError(#[from] logger::error::LoggerError),
  #[error(transparent)]
  RpcError(#[from] vino_rpc::Error),
  #[error(transparent)]
  CodecError(#[from] CodecError),
  #[error(transparent)]
  GrpcError(#[from] tonic::Status),
  #[error(transparent)]
  WascapError(#[from] vino_wascap::error::ClaimsError),
  #[error(transparent)]
  VinoHostError(#[from] vino_host::Error),
  #[error(transparent)]
  VinoRuntimeError(#[from] vino_runtime::Error),
  #[error(transparent)]
  ConnectionError(#[from] tonic::transport::Error),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
  #[error(transparent)]
  SerdeJsonError(#[from] serde_json::Error),
  #[error("General error : {0}")]
  Other(String),
}

impl From<nkeys::error::Error> for ControlError {
  fn from(e: nkeys::error::Error) -> Self {
    ControlError::KeyPairError(e.to_string())
  }
}
