use thiserror::Error;

/// The RPC Error type.
#[derive(Error, Debug)]
pub enum RpcError {
  /// Error during the parsing of an IP address and port.
  #[error(transparent)]
  AddrParseError(#[from] std::net::AddrParseError),

  /// Error parsing a UUID.
  #[error("Could not parse UUID '{0}': {1}")]
  UuidParseError(String, uuid::Error),

  /// Upstream error from Tonic.
  #[error(transparent)]
  TransportError(#[from] tonic::transport::Error),

  /// Internal Error.
  #[error("Internal Error: {0}")]
  InternalError(String),

  /// No inherent data found in RPC message.
  #[error("No inherent data found in RPC message")]
  NoInherentData,

  /// Conversion error between types.
  #[error("Could not convert type to or from gRPC to wick.")]
  TypeConversion,

  /// Invalid [crate::rpc::component::ComponentKind].
  #[error("Invalid component kind {0}")]
  InvalidComponentKind(i32),

  /// Error used by components.
  #[error("{0}")]
  Component(String),

  /// Component did not include a supported feature list.
  #[error("Component did not include a supported feature list.")]
  MissingFeatures,

  /// Error generated by a component's operations.
  #[error("{0}")]
  Operation(String),

  /// Error sending output to channel.
  #[error("Error sending output to channel")]
  SendError,

  /// General Error.
  #[error("General error : {0}")]
  General(String),

  /// Deserialization Failed.
  #[error("Deserialization Failed : {0}")]
  Deserialization(String),

  /// Error caused by an internal inconsistency.
  #[error("Internal Error : {0}")]
  Internal(&'static str),

  /// Configuration for invocation was empty.
  #[error("Configuration for invocation was empty.")]
  ConfigEmpty,

  /// Configuration for invocation was empty.
  #[error("State for invocation was missing.")]
  StateMissing,

  /// Invalid Type Signature.
  #[error("Invalid signature")]
  InvalidSignature,
}

impl RpcError {
  /// Constructor for a [Box<RpcError::General>]
  pub fn boxed<T: std::fmt::Display>(msg: T) -> Box<Self> {
    Box::new(RpcError::General(msg.to_string()))
  }
}

impl From<tokio::task::JoinError> for RpcError {
  fn from(e: tokio::task::JoinError) -> Self {
    RpcError::InternalError(format!("Tokio Error: {}", e))
  }
}

impl From<std::io::Error> for RpcError {
  fn from(e: std::io::Error) -> Self {
    RpcError::InternalError(format!("IO Error: {}", e))
  }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for RpcError {
  fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
    RpcError::Component(e.to_string())
  }
}

impl From<&str> for RpcError {
  fn from(e: &str) -> Self {
    RpcError::General(e.to_owned())
  }
}

impl From<String> for RpcError {
  fn from(e: String) -> Self {
    RpcError::General(e)
  }
}

/// The error type that [RpcClient] methods produce.
#[derive(thiserror::Error, Debug)]
pub enum RpcClientError {
  /// An error originating from a List RPC call.
  #[error("RPC List call failed: {0}")]
  ListCallFailed(tonic::Status),

  /// An error originating from an Invocation RPC call.
  #[error("RPC Invocation failed: {0}")]
  InvocationFailed(tonic::Status),

  /// An error originating from a Stats RPC call.
  #[error("RPC Stats call failed: {0}")]
  StatsCallFailed(tonic::Status),

  /// Invalid response from RPC call.
  #[error("RPC response invalid: {0}")]
  ResponseInvalid(String),

  /// Error converting to or from RPC data types.
  #[error(transparent)]
  ConversionFailed(RpcError),

  /// Conversion error between types.
  #[error("Could not convert type to or from gRPC to wick.")]
  TypeConversion,

  /// General IO error
  #[error("I/O error: {0}")]
  IO(std::io::Error),

  /// Error with Tls configuration
  #[error("Tls error: {0}")]
  TlsError(tonic::transport::Error),

  /// Error connecting to service
  #[error("{0}")]
  ConnectionError(String),

  /// Unspecified error connecting to service
  #[error("unspecified connection error")]
  UnspecifiedConnectionError,

  /// Connection failed
  #[error("Connection failed: {0}")]
  ConnectionFailed(String),

  /// General error
  #[error("{0}")]
  Other(String),
}

impl From<std::io::Error> for RpcClientError {
  fn from(e: std::io::Error) -> Self {
    RpcClientError::IO(e)
  }
}

#[cfg(test)]
mod test {

  use super::*;
  fn sync_send<T>()
  where
    T: Sync + Send,
  {
  }

  #[test]
  fn test_sync_send() {
    sync_send::<RpcError>();
    sync_send::<RpcClientError>();
  }
}
