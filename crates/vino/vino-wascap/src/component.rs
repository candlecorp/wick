use serde::{Deserialize, Serialize};
use wascap::jwt::WascapEntity;
use wasmflow_interface::ProviderSignature;

/// The metadata that corresponds to an actor module.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ProviderClaims {
  /// A hash of the module's bytes as they exist without the embedded signature. This is stored so wascap.
  /// can determine if a WebAssembly module's bytecode has been altered after it was signed.
  #[serde(rename = "hash")]
  pub module_hash: String,

  /// List of arbitrary string tags associated with the claims.
  #[serde(rename = "tags", skip_serializing_if = "Option::is_none")]
  pub tags: Option<Vec<String>>,

  /// The schema for the module
  #[serde(rename = "interface")]
  pub interface: ProviderSignature,

  /// Indicates a monotonically increasing revision number.  Optional.
  #[serde(rename = "rev", skip_serializing_if = "Option::is_none")]
  pub rev: Option<u32>,

  /// Indicates a human-friendly version string.
  #[serde(rename = "ver", skip_serializing_if = "Option::is_none")]
  pub ver: Option<String>,
}

impl WascapEntity for ProviderClaims {
  fn name(&self) -> String {
    self.interface.name.as_ref().unwrap_or(&"Anonymous".to_owned()).clone()
  }
}
