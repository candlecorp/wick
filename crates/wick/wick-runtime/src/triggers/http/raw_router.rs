use std::net::SocketAddr;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::future::BoxFuture;
use hyper::service::Service;
use hyper::{Body, Request, Response};
use tracing::{Instrument, Span};
use wick_packet::Entity;

use super::component_utils::{handle, respond};
use super::{HttpError, RawRouter, RouterOperation};
use crate::Runtime;

#[derive()]
#[must_use]
pub(super) struct RawComponentRouter {
  config: Arc<RouterOperation>,
  span: Span,
}

impl RawComponentRouter {
  pub(super) fn new(config: RouterOperation) -> Self {
    let span = debug_span!("http:raw", component = %config.operation);

    Self {
      config: Arc::new(config),
      span,
    }
  }
}

impl RawRouter for RawComponentRouter {
  fn handle(
    &self,
    remote_addr: SocketAddr,
    runtime: Arc<Runtime>,
    request: Request<Body>,
  ) -> BoxFuture<Result<Response<Body>, HttpError>> {
    let handler = RawHandler::new(self.config.clone(), runtime, remote_addr);
    let span = debug_span!("handling");
    span.follows_from(&self.span);

    let fut = async move {
      let response = handler
        .serve(request)
        .instrument(span)
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
  remote_addr: SocketAddr,
}

impl RawHandler {
  fn new(config: Arc<RouterOperation>, engine: Arc<Runtime>, remote_addr: SocketAddr) -> Self {
    RawHandler {
      config,
      engine,
      remote_addr,
    }
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
      self.remote_addr,
      self.config.config.clone(),
      &self.config.path,
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
