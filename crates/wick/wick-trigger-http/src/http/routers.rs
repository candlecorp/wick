use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use url::Url;
use wick_config::config::{BoundIdentifier, Codec};
use wick_packet::RuntimeConfig;
use wick_trigger::error::{Error, ErrorKind};
use wick_trigger::resources::{Resource, ResourceKind};

use super::RawRouter;

pub(super) mod proxy;
pub(super) mod raw;
pub(super) mod rest;
pub(super) mod static_;

#[derive(Debug, Clone)]
pub(crate) enum HttpRouter {
  Raw(RawRouterHandler),
}

impl HttpRouter {
  pub(crate) fn path(&self) -> &str {
    match self {
      HttpRouter::Raw(r) => &r.path,
    }
  }
}

#[derive(Clone)]
pub(crate) struct RawRouterHandler {
  pub(super) path: String,
  pub(super) component: Arc<dyn RawRouter + Send + Sync>,
  pub(super) middleware: super::middleware::RouterMiddleware,
}
impl std::fmt::Debug for RawRouterHandler {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("RawRouterHandler").field("path", &self.path).finish()
  }
}

#[derive(Debug, Clone)]
pub(super) struct RouterOperation {
  operation: String,
  component: String,
  codec: Codec,
  config: Option<RuntimeConfig>,
  path: String,
}

fn get_url(resources: Arc<HashMap<BoundIdentifier, Resource>>, id: &BoundIdentifier) -> Result<Url, Error> {
  let url = resources
    .get(id)
    .ok_or_else(|| -> Error { Error::new_context("http", ErrorKind::ResourceNotFound(id.clone())) })?;
  match url {
    Resource::Url(s) => Ok(s.clone()),
    _ => Err(Error::new_context(
      "http",
      ErrorKind::InvalidResourceType(ResourceKind::Url, url.kind()),
    )),
  }
}
