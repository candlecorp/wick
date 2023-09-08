use thiserror::Error;
use wick_interface_types::Type;

#[derive(Error, Debug)]
/// The error returned by the collection CLI.
#[non_exhaustive]
pub enum CliError {
  #[error(transparent)]
  #[cfg(any(feature = "grpc", feature = "mesh"))]
  /// An upstream error from [wick_rpc].
  RpcError(#[from] wick_rpc::Error),

  #[error(transparent)]
  /// An error parsing an IP address.
  IpAddrError(#[from] std::net::AddrParseError),

  #[error(transparent)]
  /// An IO error binding to a port or similar.
  IOError(#[from] std::io::Error),

  #[cfg(feature = "grpc")]
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

  #[error("invalid argument: {0}")]
  /// Thrown when the CLI received an invalid argument to pass to an invocation.
  InvalidArgument(String),

  #[error("Input '{0}' not found in signature")]
  /// Thrown when parsed input name was not found on the target operation.
  InvalidInput(String),

  #[error("{0}")]
  /// A general configuration error.
  Configuration(String),

  #[error("Could not convert data '{data}' to a format suitable for port {port}'s type {ty}")]
  /// Could not convert passed argument to a suitable intermediary format.
  Encoding {
    /// The data that could not be converted.
    data: String,
    /// The port that the data was being passed to.
    port: String,
    /// The type of the port.
    ty: Type,
  },

  #[error("Found argument '{0}' which requires a value but no value was supplied")]
  /// Dangling arguments (e.g. --arg instead of --arg value or --arg=value)
  MissingArgumentValue(String),
}

impl CliError {
  pub(crate) fn encoding<T: Into<String>, D: Into<String>>(port: T, data: D, ty: Type) -> Self {
    Self::Encoding {
      data: data.into(),
      port: port.into(),
      ty,
    }
  }
}
