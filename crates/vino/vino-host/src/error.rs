use thiserror::Error;
use vino_runtime::error::RuntimeError;

type BoxedErrorSyncSend = Box<dyn std::error::Error + Sync + Send>;
// type BoxedError = Box<dyn std::error::Error>;

#[derive(Error, Debug)]
pub enum HostError {
  #[error("invalid configuration")]
  ConfigurationError,
  #[error("File not found {0}")]
  FileNotFound(String),
  #[error("No network started yet")]
  NoNetwork,
  #[error("Configuration disallows fetching artifacts with the :latest tag ({0})")]
  LatestDisallowed(String),
  #[error("Could not fetch '{0}': {1}")]
  OciFetchFailure(String, String),
  #[error("Could not start host: {0}")]
  HostStartFailure(String),
  #[error(transparent)]
  LoadFailed(#[from] vino_loader::Error),
  #[error(transparent)]
  RuntimeError(#[from] Box<vino_runtime::Error>),
  #[error(transparent)]
  CodecError(#[from] wasmflow_codec::Error),
  #[error(transparent)]
  RpcServerError(#[from] vino_invocation_server::Error),
  #[error(transparent)]
  TransportError(#[from] vino_transport::Error),
  #[error(transparent)]
  ManifestError(#[from] vino_manifest::Error),
  #[error("Invalid host state for operation: {0}")]
  InvalidHostState(String),
  #[error("Failed to deserialize configuration {0}")]
  ConfigurationDeserialization(String),
  #[error("Async error: {0}")]
  AsyncRT(String),
  #[error(transparent)]
  KeyPairError(#[from] nkeys::error::Error),
  #[error("General error : {0}")]
  Other(String),
  #[error("{0}")]
  Lattice(String),
  #[error("Unparseable IP address: {0}")]
  BadIpAddress(String),
  #[error("Invalid file path: {0}")]
  BadPath(String),
}

impl From<BoxedErrorSyncSend> for HostError {
  fn from(e: BoxedErrorSyncSend) -> Self {
    HostError::Other(e.to_string())
  }
}

impl From<String> for HostError {
  fn from(e: String) -> Self {
    HostError::Other(e)
  }
}

impl From<&'static str> for HostError {
  fn from(e: &'static str) -> Self {
    HostError::Other(e.to_owned())
  }
}

impl From<RuntimeError> for HostError {
  fn from(e: RuntimeError) -> Self {
    HostError::RuntimeError(Box::new(e))
  }
}

impl From<vino_lattice::Error> for HostError {
  fn from(e: vino_lattice::Error) -> Self {
    HostError::Lattice(e.to_string())
  }
}
