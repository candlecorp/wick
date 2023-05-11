use std::net::SocketAddr;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::future::BoxFuture;
use hyper::service::Service;
use hyper::{Body, Request, Response};
use wick_packet::Entity;

use super::component_utils::{handle, respond};
use super::{HttpError, RawRouter, RouterOperation};
use crate::Runtime;

static ID: &str = "wick:http:raw";

#[derive()]
#[must_use]
pub(super) struct RawComponentRouter {
  config: Arc<RouterOperation>,
}

impl RawComponentRouter {
  pub(super) fn new(config: RouterOperation) -> Self {
    debug!(component = %config.operation, "{}: serving", ID);

    Self {
      config: Arc::new(config),
    }
  }
}

impl RawRouter for RawComponentRouter {
  fn handle(
    &self,
    _remote_addr: SocketAddr,
    runtime: Arc<Runtime>,
    request: Request<Body>,
  ) -> BoxFuture<Result<Response<Body>, HttpError>> {
    let handler = RawHandler::new(self.config.clone(), runtime);
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

#[derive(Clone)]
struct RawHandler {
  config: Arc<RouterOperation>,
  engine: Arc<Runtime>,
}

impl RawHandler {
  fn new(config: Arc<RouterOperation>, engine: Arc<Runtime>) -> Self {
    RawHandler { config, engine }
  }

  /// Serve a request.
  async fn serve(self, req: Request<Body>) -> Result<Response<Body>, HttpError> {
    let config = self.config.clone();
    let engine = self.engine.clone();
    let codec = config.codec;
    let stream = handle(
      Entity::operation(&config.component, &config.operation),
      config.codec,
      engine,
      req,
    )
    .await;
    respond(codec, stream).await
  }
}

impl Service<Request<Body>> for RawHandler {
  type Response = Response<Body>;
  type Error = HttpError;
  type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

  fn poll_ready(&mut self, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, request: Request<Body>) -> Self::Future {
    Box::pin(self.clone().serve(request))
  }
}
