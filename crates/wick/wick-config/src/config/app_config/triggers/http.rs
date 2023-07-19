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

#[derive(Debug, Clone, derive_asset_container::AssetManager, property::Property, serde::Serialize, Builder)]
#[builder(setter(into))]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(AssetReference))]
#[must_use]
pub struct HttpTriggerConfig {
  #[asset(skip)]
  pub(crate) resource: String,
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) routers: Vec<HttpRouterConfig>,
}

#[derive(Debug, Clone, derive_asset_container::AssetManager, serde::Serialize)]
#[asset(asset(AssetReference))]
#[must_use]
#[serde(rename_all = "kebab-case")]
pub enum HttpRouterConfig {
  RawRouter(RawRouterConfig),
  RestRouter(RestRouterConfig),
  StaticRouter(StaticRouterConfig),
  ProxyRouter(ProxyRouterConfig),
}

impl HttpRouterConfig {
  #[must_use]
  pub fn kind(&self) -> HttpRouterKind {
    match self {
      Self::RawRouter(_) => HttpRouterKind::RawRouter,
      Self::RestRouter(_) => HttpRouterKind::RestRouter,
      Self::StaticRouter(_) => HttpRouterKind::StaticRouter,
      Self::ProxyRouter(_) => HttpRouterKind::ProxyRouter,
    }
  }

  #[must_use]
  pub fn path(&self) -> &str {
    match self {
      Self::RawRouter(r) => r.path(),
      Self::RestRouter(r) => r.path(),
      Self::StaticRouter(r) => r.path(),
      Self::ProxyRouter(r) => r.path(),
    }
  }
}

pub trait WickRouter {
  fn middleware(&self) -> Option<&Middleware>;
  fn path(&self) -> &str;
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HttpRouterKind {
  RawRouter,
  RestRouter,
  StaticRouter,
  ProxyRouter,
}

impl std::fmt::Display for HttpRouterKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::RawRouter => write!(f, "raw"),
      Self::RestRouter => write!(f, "rest"),
      Self::StaticRouter => write!(f, "static"),
      Self::ProxyRouter => write!(f, "proxy"),
    }
  }
}
