#![allow(missing_docs)] // delete when we move away from the `property` crate.

use crate::config::LiquidJsonConfig;

/// A component exposed as an external microservice.
#[derive(Debug, Clone, PartialEq, property::Property)]
#[property(get(public), set(private), mut(disable))]
pub struct GrpcUrlComponent {
  /// The URL to connect to.
  pub(crate) url: String,
  /// The configuration for the component
  pub(crate) config: Option<LiquidJsonConfig>,
}
