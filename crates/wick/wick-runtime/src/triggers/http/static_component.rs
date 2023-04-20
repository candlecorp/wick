use std::net::SocketAddr;

use futures::future::BoxFuture;
use hyper::{Body, Request, Response};
use hyper_staticfile::Static;

use super::{HttpError, RawRouter};

#[derive()]
#[must_use]
pub(super) struct StaticComponent {
  handler: Static,
}

impl StaticComponent {
  pub(super) fn new(root: String) -> Self {
    let handler = hyper_staticfile::Static::new(root);
    Self { handler }
  }
}

impl RawRouter for StaticComponent {
  fn handle(&self, _remote_addr: SocketAddr, request: Request<Body>) -> BoxFuture<Result<Response<Body>, HttpError>> {
    let handler = self.handler.clone();
    let fut = async move {
      let response = handler
        .serve(request)
        .await
        .map_err(|e| HttpError::OperationError(e.to_string()))?;
      Ok(response)
    };
    Box::pin(fut)
  }
}
