use wick_asset_reference::AssetReference;

#[derive(Debug, Clone, derive_asset_container::AssetManager, property::Property)]
#[asset(asset(AssetReference))]
#[property(get(public), set(private), mut(disable))]

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
