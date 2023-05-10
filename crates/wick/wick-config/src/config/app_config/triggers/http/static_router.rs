use wick_asset_reference::AssetReference;

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]
#[must_use]
pub struct StaticRouterConfig {
  #[asset(skip)]
  pub(crate) path: String,
  #[asset(skip)]
  pub(crate) volume: String,
}

impl StaticRouterConfig {
  /// Returns the path for the static router.
  #[must_use]
  pub fn path(&self) -> &str {
    &self.path
  }
  /// Returns the volume name for the static router.
  #[must_use]
  pub fn volume(&self) -> &str {
    &self.volume
  }
}
