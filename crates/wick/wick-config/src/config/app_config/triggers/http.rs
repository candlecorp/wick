use wick_asset_reference::AssetReference;

use crate::config::{ComponentDefinition, ComponentOperationExpression};

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]
#[must_use]
pub struct HttpTriggerConfig {
  #[asset(skip)]
  pub(crate) resource: String,
  pub(crate) routers: Vec<HttpRouterConfig>,
}

impl HttpTriggerConfig {
  #[must_use]
  pub fn resource_id(&self) -> &str {
    &self.resource
  }
  pub fn routers(&self) -> &[HttpRouterConfig] {
    &self.routers
  }
}

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]
#[must_use]
pub struct RawRouterConfig {
  #[asset(skip)]
  pub(crate) path: String,
  pub(crate) operation: ComponentOperationExpression,
}

impl RawRouterConfig {
  #[must_use]
  pub fn path(&self) -> &str {
    &self.path
  }
  #[must_use]
  pub fn operation(&self) -> &ComponentOperationExpression {
    &self.operation
  }
}

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]
#[must_use]
pub struct RestRouterConfig {
  #[asset(skip)]
  pub(crate) path: String,
  pub(crate) component: ComponentDefinition,
}

impl RestRouterConfig {
  #[must_use]
  pub fn path(&self) -> &str {
    &self.path
  }
  pub fn component(&self) -> &ComponentDefinition {
    &self.component
  }
}

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

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]
#[must_use]
pub enum HttpRouterConfig {
  RawRouter(RawRouterConfig),
  RestRouter(RestRouterConfig),
  StaticRouter(StaticRouterConfig),
  ProxyRouter(ProxyRouterConfig),
}

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
