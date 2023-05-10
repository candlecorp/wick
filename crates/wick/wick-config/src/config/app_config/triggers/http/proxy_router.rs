use wick_asset_reference::AssetReference;

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]
pub struct ProxyRouterConfig {
  /// The path to start serving this router from.
  #[asset(skip)]
  pub(crate) path: String,
  /// The URL resource to proxy to.
  #[asset(skip)]
  pub(crate) url: String,
  /// Whether or not to strip the router's path from the proxied request.
  #[asset(skip)]
  pub(crate) strip_path: bool,
}

impl ProxyRouterConfig {
  /// Returns the path the proxy router is configured to start proxying from.
  #[must_use]
  pub fn path(&self) -> &str {
    &self.path
  }
  /// Returns the URL for the proxy router.
  #[must_use]
  pub fn url(&self) -> &str {
    &self.url
  }
  /// Returns whether or not to strip the router's path from the proxied request.
  #[must_use]
  pub fn strip_path(&self) -> bool {
    self.strip_path
  }
}
