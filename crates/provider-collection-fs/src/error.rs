use thiserror::Error;
type BoxedError = Box<dyn std::error::Error + Sync + Send>;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Deserialization error {0}")]
  RpcMessageError(&'static str),
  #[error("Client is shutting down, streams are closing")]
  ShuttingDown,
  #[error("Error {0}")]
  Other(String),
  #[error(transparent)]
  RpcError(#[from] vino_rpc::Error),
  #[error(transparent)]
  CliError(#[from] vino_provider_cli::Error),
  #[error(transparent)]
  VinoError(#[from] vino_runtime::Error),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error(transparent)]
  ProviderSdkError(#[from] vino_provider::Error),
  #[error(transparent)]
  ComponentError(#[from] vino_component::Error),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error(transparent)]
  UpstreamError(#[from] BoxedError),
  #[error(transparent)]
  JoinError(#[from] tokio::task::JoinError),
}
