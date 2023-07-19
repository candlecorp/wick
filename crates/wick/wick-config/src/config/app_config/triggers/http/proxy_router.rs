use wick_asset_reference::AssetReference;

#[derive(Debug, Clone, derive_asset_container::AssetManager, property::Property, serde::Serialize)]
#[asset(asset(AssetReference))]
#[property(get(public), set(private), mut(disable))]

pub struct ProxyRouterConfig {
  /// The path to start serving this router from.
  #[asset(skip)]
  #[property(get(disable))]
  pub(crate) path: String,
  /// Middleware operations for this router.
  #[property(get(disable))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) middleware: Option<super::middleware::Middleware>,
  /// The URL resource to proxy to.
  #[asset(skip)]
  pub(crate) url: String,
  /// Whether or not to strip the router's path from the proxied request.
  #[asset(skip)]
  pub(crate) strip_path: bool,
}

impl super::WickRouter for ProxyRouterConfig {
  fn middleware(&self) -> Option<&super::Middleware> {
    self.middleware.as_ref()
  }

  fn path(&self) -> &str {
    &self.path
  }
}
