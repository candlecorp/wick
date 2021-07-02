use thiserror::Error;

#[derive(Error, Debug)]
pub enum RpcError {
  #[error(transparent)]
  IpAddrError(#[from] std::net::AddrParseError),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
  #[error(transparent)]
  TransportError(#[from] tonic::transport::Error),
  #[error(transparent)]
  JoinError(#[from] tokio::task::JoinError),
  #[error(transparent)]
  EntityError(#[from] vino_entity::Error),
  #[error("Invalid output kind {0}")]
  InvalidOutputKind(i32),
  #[error("General error : {0}")]
  Other(String),
}
