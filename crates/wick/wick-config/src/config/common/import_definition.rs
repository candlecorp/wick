use crate::config;

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager)]
#[asset(asset(config::AssetReference))]
/// The kinds of collections that can operate in a flow.
#[must_use]
pub enum ImportDefinition {
  /// A wick component.
  Component(config::ComponentDefinition),
  /// A type manifest.
  Types(config::components::TypesComponent),
}

impl ImportDefinition {
  /// Returns true if the definition is a reference to another component.
  #[must_use]
  pub fn is_reference(&self) -> bool {
    if let ImportDefinition::Component(c) = self {
      return c.is_reference();
    }
    false
  }
}
