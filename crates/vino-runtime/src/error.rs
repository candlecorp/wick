use thiserror::Error;

type BoxedErrorSyncSend = Box<dyn std::error::Error + Sync + Send>;
// type BoxedError = Box<dyn std::error::Error>;

#[derive(Error, Debug)]
pub enum VinoError {
  #[error("Conversion error")]
  ConversionError,
  #[error("Network error: {0}")]
  NetworkError(String),
  #[error("Schematic error: {0}")]
  SchematicError(String),
  #[error("Dispatch error: {0}")]
  DispatchError(String),
  #[error("Provider error {0}")]
  ProviderError(String),

  #[error("Component error: {0}")]
  ComponentError(String),
  #[error("Job error: {0}")]
  JobError(String),
  #[error("invalid configuration")]
  ConfigurationError,
  #[error("File not found {0}")]
  FileNotFound(String),
  #[error("Configuration disallows fetching artifacts with the :latest tag ({0})")]
  LatestDisallowed(String),
  #[error("Could not fetch '{0}': {1}")]
  OciFetchFailure(String, String),
  #[error("Could not start host: {0}")]
  HostStartFailure(String),
  #[error("Failed to deserialize configuration {0}")]
  ConfigurationDeserialization(String),
  #[error("Failed to serialize payload {0}")]
  SerializationError(rmp_serde::encode::Error),
  #[error("Failed to deserialize payload {0}")]
  DeserializationError(rmp_serde::decode::Error),
  #[error(transparent)]
  RpcUpstreamError(#[from] tonic::Status),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error(transparent)]
  TransportError(#[from] vino_transport::Error),
  #[error(transparent)]
  YamlError(#[from] serde_yaml::Error),
  #[error(transparent)]
  ActixMailboxError(#[from] actix::MailboxError),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
  #[error(transparent)]
  KeyPairError(#[from] nkeys::error::Error),
  #[error(transparent)]
  WascapError(#[from] wascap::Error),
  #[error(transparent)]
  BCryptError(#[from] bcrypt::BcryptError),
  #[error("Could not parse OCI URL: {0}")]
  OCIParseError(String),
  #[error(transparent)]
  OtherUpstream(#[from] BoxedErrorSyncSend),
  #[error("General error : {0}")]
  Other(String),
}

impl From<&'static str> for VinoError {
  fn from(e: &'static str) -> Self {
    VinoError::Other(e.to_string())
  }
}
