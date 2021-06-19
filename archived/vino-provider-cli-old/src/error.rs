use std::net::Ipv4Addr;

use thiserror::Error;

type BoxedSyncSendError = Box<dyn std::error::Error + Sync + std::marker::Send>;

#[derive(Error, Debug)]
pub enum CliError {
  #[error(transparent)]
  VinoError(#[from] vino_runtime::Error),
  #[error(transparent)]
  IpAddrError(#[from] std::net::AddrParseError),
  #[error(transparent)]
  RpcError(#[from] vino_rpc::Error),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
  // #[error(transparent)]
  // JoinError(#[from] tokio::task::JoinError),
  #[error(transparent)]
  JoinError(#[from] tokio::task::JoinError),
  #[error("General error : {0}")]
  Other(String),
}
