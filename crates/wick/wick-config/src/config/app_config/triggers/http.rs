use std::collections::HashMap;
use std::path::Path;

pub use middleware::{Middleware, MiddlewareBuilder, MiddlewareBuilderError};
use wick_asset_reference::AssetReference;
use wick_packet::RuntimeConfig;

pub use self::proxy_router::{ProxyRouterConfig, ProxyRouterConfigBuilder, ProxyRouterConfigBuilderError};
pub use self::raw_router::{RawRouterConfig, RawRouterConfigBuilder, RawRouterConfigBuilderError};
pub use self::rest_router::{
  Contact,
  Documentation,
  Info,
  License,
  RestRoute,
  RestRouterConfig,
  RestRouterConfigBuilder,
  RestRouterConfigBuilderError,
  Tools,
};
pub use self::static_router::{StaticRouterConfig, StaticRouterConfigBuilder, StaticRouterConfigBuilderError};
use crate::config::bindings::BoundIdentifier;
use crate::config::common::template_config::Renderable;
use crate::config::{Binding, ImportDefinition};
use crate::error::ManifestError;
use crate::ExpandImports;

mod middleware;
mod proxy_router;
mod raw_router;
mod rest_router;
mod static_router;

fn index_to_router_id(trigger_index: usize, index: usize) -> String {
  format!("trigger_{}_router_{}", trigger_index, index)
}
#[derive(
  Debug, Clone, derive_asset_container::AssetManager, property::Property, serde::Serialize, derive_builder::Builder,
)]
#[builder(setter(into))]
#[property(get(public), set(private), mut(public, suffix = "_mut"))]
#[asset(asset(AssetReference))]
#[must_use]
pub struct HttpTriggerConfig {
  #[asset(skip)]
  pub(crate) resource: BoundIdentifier,
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

impl Renderable for HttpRouterConfig {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    match self {
      HttpRouterConfig::RawRouter(v) => v.render_config(source, root_config, env),
      HttpRouterConfig::RestRouter(v) => v.render_config(source, root_config, env),
      HttpRouterConfig::StaticRouter(v) => v.render_config(source, root_config, env),
      HttpRouterConfig::ProxyRouter(v) => v.render_config(source, root_config, env),
    }
  }
}

impl Renderable for HttpTriggerConfig {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.routers.render_config(source, root_config, env)
  }
}

impl ExpandImports for HttpTriggerConfig {
  type Error = ManifestError;
  fn expand_imports(
    &mut self,
    bindings: &mut Vec<Binding<ImportDefinition>>,
    trigger_index: usize,
  ) -> Result<(), Self::Error> {
    for (router_index, router) in self.routers_mut().iter_mut().enumerate() {
      match router {
        HttpRouterConfig::RawRouter(r) => raw_router::process_runtime_config(trigger_index, router_index, r, bindings)?,
        HttpRouterConfig::StaticRouter(r) => {
          static_router::process_runtime_config(trigger_index, router_index, r, bindings)?;
        }
        HttpRouterConfig::ProxyRouter(r) => {
          proxy_router::process_runtime_config(trigger_index, router_index, r, bindings)?;
        }
        HttpRouterConfig::RestRouter(r) => {
          rest_router::process_runtime_config(trigger_index, router_index, r, bindings)?;
        }
      };
    }

    Ok(())
  }
}

impl HttpRouterConfig {
  #[must_use]
  pub const fn kind(&self) -> HttpRouterKind {
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
  fn middleware_mut(&mut self) -> Option<&mut Middleware>;
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
