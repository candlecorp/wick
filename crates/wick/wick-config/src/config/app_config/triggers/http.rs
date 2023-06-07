pub use middleware::Middleware;
use wick_asset_reference::AssetReference;

pub use self::proxy_router::ProxyRouterConfig;
pub use self::raw_router::RawRouterConfig;
pub use self::rest_router::*;
pub use self::static_router::StaticRouterConfig;

mod middleware;
mod proxy_router;
mod raw_router;
mod rest_router;
mod static_router;

#[derive(Debug, Clone, derive_asset_container::AssetManager, property::Property, Builder)]
#[builder(setter(into))]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(AssetReference))]
#[must_use]
pub struct HttpTriggerConfig {
  #[asset(skip)]
  pub(crate) resource: String,
  pub(crate) routers: Vec<HttpRouterConfig>,
}

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]
#[must_use]
pub enum HttpRouterConfig {
  RawRouter(RawRouterConfig),
  RestRouter(RestRouterConfig),
  StaticRouter(StaticRouterConfig),
  ProxyRouter(ProxyRouterConfig),
}

pub trait WickRouter {
  fn middleware(&self) -> Option<&Middleware>;
  fn path(&self) -> &str;
}
