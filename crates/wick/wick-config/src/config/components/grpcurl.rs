#![allow(missing_docs)] // delete when we move away from the `property` crate.

use crate::config::LiquidJsonConfig;

/// A component exposed as an external microservice.
#[derive(Debug, Clone, PartialEq, property::Property, serde::Serialize)]
#[property(get(public), set(private), mut(disable))]
pub struct GrpcUrlComponent {
  /// The URL to connect to.
  pub(crate) url: String,
  /// The configuration for the component
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) config: Option<LiquidJsonConfig>,
}
