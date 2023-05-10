use std::net::SocketAddr;

use futures::future::BoxFuture;
use hyper::{Body, Request, Response};
use wick_config::config::RestRouterConfig;

use super::{HttpError, RawRouter};

static ID: &str = "wick:http:rest";

#[derive()]
#[must_use]
pub(super) struct RestRouter {
  #[allow(unused)]
  config: RestRouterConfig,
}

impl RestRouter {
  pub(super) fn new(config: RestRouterConfig) -> Self {
    let title = config
      .info()
      .and_then(|i| i.title().cloned())
      .unwrap_or_else(|| "Untitled API".to_owned());
    debug!(api = %title, "{}: serving", ID);
    Self { config }
  }
}

impl RawRouter for RestRouter {
  fn handle(&self, _remote_addr: SocketAddr, _request: Request<Body>) -> BoxFuture<Result<Response<Body>, HttpError>> {
    todo!()
  }
}
