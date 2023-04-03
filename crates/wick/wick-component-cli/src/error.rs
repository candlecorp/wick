use thiserror::Error;

#[derive(Error, Debug)]
/// The error returned by the collection CLI.
pub enum CliError {
  #[error(transparent)]
  #[cfg(any(feature = "grpc", feature = "mesh"))]
  /// An upstream error from [wick_rpc].
  VinoError(#[from] wick_rpc::Error),

  #[error(transparent)]
  /// An error parsing an IP address.
  IpAddrError(#[from] std::net::AddrParseError),

  #[error(transparent)]
  /// An error from the logger.
  LoggerError(#[from] logger::error::LoggerError),

  #[error(transparent)]
  /// An IO error binding to a port or similar.
  IOError(#[from] std::io::Error),

  #[error(transparent)]
  /// Error related to configuration or asset loading.
  Config(#[from] wick_config::AssetError),

  #[error(transparent)]
  #[cfg(feature = "grpc")]
  /// An upstream error from [tonic].
  TransportError(#[from] tonic::transport::Error),

  #[error(transparent)]
  /// An internal error running asynchronous jobs.
  JoinError(#[from] tokio::task::JoinError),

  #[error("{0}")]
  /// Thrown when the CLI received an invalid argument to pass to an invocation.
  InvalidArgument(String),

  #[error("{0}")]
  /// A general configuration error.
  Configuration(String),

  #[error("Found argument '{0}' which requires a value but no value was supplied")]
  /// Dangling arguments (e.g. --arg instead of --arg value or --arg=value)
  MissingArgumentValue(String),
}
