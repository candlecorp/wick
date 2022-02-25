use thiserror::Error;
use vino_codec::error::CodecError;

#[derive(Error, Debug)]
pub(crate) enum ControlError {
  #[error("Keypair error: {0}")]
  KeyPairError(String),
  #[error("Keypair path or string not supplied. Ensure provided keypair is valid")]
  KeyPairNotProvided,
  #[error(transparent)]
  LoggerError(#[from] logger::error::LoggerError),
  #[error(transparent)]
  RpcClientError(#[from] vino_rpc::error::RpcClientError),
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
  TransportError(#[from] vino_transport::Error),
  #[error("Connection failed: {0}")]
  ConnectionError(String),
  #[error("Internal error: {0}")]
  InternalError(String),
  #[error("Could not read or open file: {0}")]
  ReadFailed(std::io::Error),
  #[error("Could not read next line: {0}")]
  ReadLineFailed(std::io::Error),
  #[error("{0}")]
  CliError(String),
  #[error("IO error: {0}")]
  IOError(String),
  #[error(transparent)]
  SerdeJsonError(#[from] serde_json::Error),
  #[error(transparent)]
  PackFailed(#[from] vino_par::Error),

  #[error(transparent)]
  OCIError(#[from] vino_oci::Error),
}

impl From<nkeys::error::Error> for ControlError {
  fn from(e: nkeys::error::Error) -> Self {
    ControlError::KeyPairError(e.to_string())
  }
}

impl From<std::io::Error> for ControlError {
  fn from(e: std::io::Error) -> Self {
    ControlError::IOError(e.to_string())
  }
}

impl From<vino_provider_cli::Error> for ControlError {
  fn from(e: vino_provider_cli::Error) -> Self {
    ControlError::CliError(e.to_string())
  }
}

// TODO: Submit PRs to improve tonic's error handling
impl From<tonic::transport::Error> for ControlError {
  fn from(e: tonic::transport::Error) -> Self {
    let debug = format!("Tonic error: {:?}", e);
    if debug.contains("Connection refused") {
      Self::ConnectionError("Connection refused".to_owned())
    } else {
      Self::InternalError("Internal error: TONIC".to_owned())
    }
  }
}
