use serde::{
  Deserialize,
  Serialize,
};
use vino_rpc::ProviderSignature;

/// The metadata that corresponds to an actor module
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ComponentClaims {
  /// A descriptive name for this actor, should not include version information or public key
  #[serde(rename = "ver", skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,

  /// A hash of the module's bytes as they exist without the embedded signature. This is stored so wascap
  /// can determine if a WebAssembly module's bytecode has been altered after it was signed
  #[serde(rename = "hash")]
  pub module_hash: String,

  /// List of arbitrary string tags associated with the claims
  #[serde(rename = "tags", skip_serializing_if = "Option::is_none")]
  pub tags: Option<Vec<String>>,

  /// List of input ports.
  #[serde(rename = "interface")]
  pub interface: ProviderSignature,

  /// Indicates a monotonically increasing revision number.  Optional.
  #[serde(rename = "rev", skip_serializing_if = "Option::is_none")]
  pub rev: Option<i32>,

  /// Indicates a human-friendly version string
  #[serde(rename = "ver", skip_serializing_if = "Option::is_none")]
  pub ver: Option<String>,
}
