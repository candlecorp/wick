use wick_asset_reference::AssetReference;

#[derive(Debug, Clone, derive_asset_container::AssetManager, property::Property)]
#[asset(asset(AssetReference))]
#[property(get(public), set(private), mut(disable))]
#[must_use]
pub struct StaticRouterConfig {
  #[asset(skip)]
  #[property(get(disable))]
  pub(crate) path: String,
  /// Middleware operations for this router.
  #[property(get(disable))]
  pub(crate) middleware: Option<super::middleware::Middleware>,
  #[asset(skip)]
  pub(crate) volume: String,
  #[asset(skip)]
  pub(crate) fallback: Option<String>,
}

impl super::WickRouter for StaticRouterConfig {
  fn middleware(&self) -> Option<&super::Middleware> {
    self.middleware.as_ref()
  }

  fn path(&self) -> &str {
    &self.path
  }
}
