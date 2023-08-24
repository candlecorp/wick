use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::future::BoxFuture;
use hyper::service::Service;
use hyper::{Body, Method, Request, Response};
use hyper_staticfile::{resolve_path, ResolveResult, ResponseBuilder};
use tracing::{Instrument, Span};
use uuid::Uuid;
use wick_config::config::{StaticRouterConfig, TriggerKind, WickRouter};

use crate::dev::prelude::RuntimeError;
use crate::resources::{Resource, ResourceKind};
use crate::triggers::http::middleware::resolve_middleware_components;
use crate::triggers::http::{HttpError, HttpRouter, RawRouter, RawRouterHandler};
use crate::Runtime;

#[derive()]
#[must_use]
pub(super) struct StaticRouter {
  handler: Static,
}

impl StaticRouter {
  pub(super) fn new(root: PathBuf, strip: Option<String>, fallback: Option<String>) -> Self {
    debug!(directory = %root.display(), "http:static:serving");
    let handler = Static::new(root, strip, fallback);
    Self { handler }
  }
}

impl RawRouter for StaticRouter {
  fn handle(
    &self,
    _tx_id: Uuid,
    _remote_addr: SocketAddr,
    _runtime: Runtime,
    request: Request<Body>,
    span: &Span,
  ) -> BoxFuture<Result<Response<Body>, HttpError>> {
    let span = info_span!(parent: span, "static");
    let handler = self.handler.clone();
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

fn create_response<B>(request: &Request<B>, result: ResolveResult) -> Result<Response<Body>, std::io::Error>
where
  B: Send + Sync + 'static,
{
  #[allow(clippy::expect_used)]
  Ok(
    ResponseBuilder::new()
      .request(request)
      .build(result)
      .expect("unable to build response"),
  )
}

#[derive(Clone)]
struct Static {
  root: PathBuf,
  strip: Option<String>,
  fallback: Option<String>,
}

impl Static {
  fn new(root: impl Into<PathBuf>, strip: Option<String>, fallback: Option<String>) -> Self {
    let root = root.into();
    Static { root, strip, fallback }
  }

  /// Serve a request.
  async fn serve<B>(self, request: Request<B>) -> Result<Response<Body>, std::io::Error>
  where
    B: Send + Sync + 'static,
  {
    let Self { root, strip, fallback } = self;
    // Handle only `GET`/`HEAD` and absolute paths.
    match *request.method() {
      Method::HEAD | Method::GET => {}
      _ => {
        return create_response(&request, ResolveResult::MethodNotMatched);
      }
    }

    let path = strip.map_or_else(
      || request.uri().path(),
      |path| {
        if path.len() > 1 {
          request.uri().path().trim_start_matches(&path)
        } else {
          request.uri().path()
        }
      },
    );

    let result = resolve_path(root.clone(), path).await;

    match result {
      Ok(ResolveResult::Found(_, _, _)) => create_response(&request, result?),
      _ => {
        if let Some(fb) = &fallback {
          let fallback_result = resolve_path(root.clone(), fb).await;
          create_response(&request, fallback_result?)
        } else {
          create_response(&request, result?)
        }
      }
    }
  }
}

impl<B: Send + Sync + 'static> Service<Request<B>> for Static {
  type Response = Response<Body>;
  type Error = std::io::Error;
  type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

  fn poll_ready(&mut self, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, request: Request<B>) -> Self::Future {
    Box::pin(self.clone().serve(request))
  }
}

pub(crate) fn register_static_router(
  index: usize,
  resources: Arc<HashMap<String, Resource>>,
  router_config: &StaticRouterConfig,
) -> Result<HttpRouter, RuntimeError> {
  trace!(index, "registering static router");
  let middleware = resolve_middleware_components(router_config)?;
  let volume = resources.get(router_config.volume()).ok_or_else(|| {
    RuntimeError::ResourceNotFound(
      TriggerKind::Http.into(),
      format!("volume {} not found", router_config.volume()),
    )
  })?;
  let volume = match volume {
    Resource::Volume(s) => s.clone(),
    _ => {
      return Err(RuntimeError::InvalidResourceType(
        TriggerKind::Http.into(),
        ResourceKind::Volume,
        volume.kind(),
      ))
    }
  };

  let fallback = router_config.fallback().cloned();

  let router = StaticRouter::new(volume, Some(router_config.path().to_owned()), fallback);

  Ok(HttpRouter::Raw(RawRouterHandler {
    path: router_config.path().to_owned(),
    component: Arc::new(router),
    middleware,
  }))
}
