/// A collection exposed as an external microservice.
#[derive(Debug, Clone, PartialEq)]
pub struct GrpcUrlComponent {
  /// The URL to connect to .
  pub url: String,
  /// The configuration for the collection
  pub config: Option<wick_packet::OperationConfig>,
}
