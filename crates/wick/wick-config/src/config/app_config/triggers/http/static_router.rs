use wick_asset_reference::AssetReference;

#[derive(Debug, Clone, derive_asset_container::AssetManager, property::Property)]
#[asset(asset(AssetReference))]
#[property(get(public), set(private), mut(disable))]
#[must_use]
pub struct StaticRouterConfig {
  #[asset(skip)]
  pub(crate) path: String,
  #[asset(skip)]
  pub(crate) volume: String,
  #[asset(skip)]
  pub(crate) fallback: Option<String>,
}
