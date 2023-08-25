use wick_asset_reference::AssetReference;

use crate::config::{ComponentOperationExpression, ImportBinding};
use crate::error::ManifestError;
use crate::ExpandImports;

#[derive(
  Debug,
  Clone,
  PartialEq,
  derive_asset_container::AssetManager,
  property::Property,
  serde::Serialize,
  derive_builder::Builder,
)]
#[builder(setter(into))]
#[asset(asset(AssetReference))]
#[property(get(public), set(private), mut(public, suffix = "_mut"))]

/// Normalized representation of a CLI trigger configuration.
pub struct CliConfig {
  pub(crate) operation: ComponentOperationExpression,
}

impl ExpandImports for CliConfig {
  type Error = ManifestError;
  fn expand_imports(&mut self, bindings: &mut Vec<ImportBinding>, trigger_index: usize) -> Result<(), Self::Error> {
    let id = format!("trigger_{}", trigger_index);
    self.operation_mut().maybe_import(&id, bindings);
    Ok(())
  }
}
