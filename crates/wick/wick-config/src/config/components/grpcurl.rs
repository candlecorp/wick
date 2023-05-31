/// A collection exposed as an external microservice.
#[derive(Debug, Clone, PartialEq, property::Property)]
#[property(get(public), set(private), mut(disable))]
pub struct GrpcUrlComponent {
  /// The URL to connect to .
  pub(crate) url: String,
  /// The configuration for the collection
  pub(crate) config: Option<wick_packet::GenericConfig>,
}
