use thiserror::Error;

#[derive(Error, Debug)]
/// The error returned by the provider CLI.
pub enum CliError {
  #[error(transparent)]
  /// An upstream error from [vino_rpc].
  VinoError(#[from] vino_rpc::Error),

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
  /// An upstream error from [tonic].
  TransportError(#[from] tonic::transport::Error),

  #[error(transparent)]
  /// An internal error running asynchronous jobs.
  JoinError(#[from] tokio::task::JoinError),

  #[error(transparent)]
  /// An error connecting or communicating over the lattice.
  Lattice(#[from] vino_lattice::Error),

  #[error("{0}")]
  /// A general configuration error.
  Configuration(String),

  #[error("Found argument '{0}' which wasn't expected, or isn't valid in this context")]
  /// Error parsing arguments into a [vino_transport::TransportMap].
  InvalidArgument(String),

  #[error("Found argument '{0}' which requires a value but no value was supplied")]
  /// Dangling arguments (e.g. --arg instead of --arg value or --arg=value)
  MissingArgumentValue(String),
}
