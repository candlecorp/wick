use wick_interface_types::{Field, OperationSignatures};

use crate::config::components::ComponentConfig;
use crate::config::{self, ExposedVolume, OperationDefinition};
use crate::utils::VecMapInto;

#[derive(
  Debug, Clone, derive_asset_container::AssetManager, derive_builder::Builder, property::Property, serde::Serialize,
)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[builder(setter(into))]
#[asset(asset(config::AssetReference))]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct WasmRsComponent {
  /// The location of the component.
  pub(crate) reference: config::AssetReference,

  /// The configuration for the component.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) config: Vec<Field>,

  /// Volumes to expose to the component and the paths they map to.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) volumes: Vec<ExposedVolume>,

  /// The operations defined by the component.
  #[asset(skip)]
  #[builder(default)]
  #[property(skip)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) operations: Vec<OperationDefinition>,

  /// The default buffer size to use for the component.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) max_packet_size: Option<u32>,
}

impl OperationSignatures for WasmRsComponent {
  fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    self.operations.clone().map_into()
  }
}

impl ComponentConfig for WasmRsComponent {
  type Operation = OperationDefinition;

  fn operations(&self) -> &[Self::Operation] {
    &self.operations
  }

  fn operations_mut(&mut self) -> &mut Vec<Self::Operation> {
    &mut self.operations
  }
}
