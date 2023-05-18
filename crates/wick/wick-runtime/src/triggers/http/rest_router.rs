use std::net::SocketAddr;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::future::BoxFuture;
use hyper::body::to_bytes;
use hyper::service::Service;
use hyper::{Body, Request, Response, StatusCode};
use tracing::{Instrument, Span};
use wick_config::config::{ComponentOperationExpression, ImportBinding, RestRouterConfig};
use wick_packet::{Entity, Invocation, Packet};
mod route;

use super::{HttpError, RawRouter};
use crate::triggers::http::component_utils::stream_to_json;
use crate::Runtime;

#[derive()]
#[must_use]
pub(super) struct RestRouter {
  routes: Arc<Vec<RestRoute>>,
  root: String,
  span: Span,
}

impl RestRouter {
  pub(super) fn new(config: RestRouterConfig, routes: Vec<RestRoute>) -> Self {
    let span = debug_span!("http:rest", path = %config.path());

    let title = config
      .info()
      .and_then(|i| i.title().cloned())
      .unwrap_or_else(|| "Untitled API".to_owned());

    let routes = span.in_scope(|| {
      debug!(api = %title, path=%config.path(), "serving");
      for route in &routes {
        debug!(route = ?route.route, "route");
      }
      Arc::new(routes)
    });

    Self {
      routes,
      root: config.path().to_owned(),
      span,
    }
  }
}

impl RawRouter for RestRouter {
  fn handle(
    &self,
    _remote_addr: SocketAddr,
    runtime: Arc<Runtime>,
    request: Request<Body>,
  ) -> BoxFuture<Result<Response<Body>, HttpError>> {
    let span = debug_span!("handling");
    span.follows_from(&self.span);
    let handler = RestHandler::new(self.root.clone(), self.routes.clone(), runtime, span);
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
struct RestHandler {
  routes: Arc<Vec<RestRoute>>,
  runtime: Arc<Runtime>,
  root: String,
  span: Span,
}

impl RestHandler {
  fn new(root: String, routes: Arc<Vec<RestRoute>>, runtime: Arc<Runtime>, span: Span) -> Self {
    RestHandler {
      runtime,
      routes,
      root,
      span,
    }
  }

  /// Serve a request.
  #[allow(clippy::unused_async)]
  async fn serve(self, request: Request<Body>) -> Result<Response<Body>, HttpError> {
    let Self {
      routes,
      runtime,
      root,
      span,
    } = self;

    for route in routes.iter() {
      if !route.config.methods().is_empty() && !route.config.methods().contains(&request.method().to_string()) {
        continue;
      }
      let path = request.uri().path().trim_start_matches(root.as_str());
      let Some((path_params, query_params)) = route.route.compare(path, request.uri().query())? else {
        continue
      };
      span.in_scope(|| debug!(route = %request.uri(), "handling"));
      let mut packets: Vec<_> = path_params
        .iter()
        .chain(query_params.iter())
        .map(|f| Packet::encode(f.name(), f.value()))
        .collect();

      let (_, body) = request.into_parts();

      let body_bytes = to_bytes(body).await.unwrap_or_default();

      let payload = match serde_json::from_slice(&body_bytes) {
        Ok(json) => json,
        Err(_e) => serde_json::json!({}),
      };
      packets.push(Packet::encode("input", payload));

      let mut port_names: Vec<_> = packets.iter().map(|p| p.port().to_owned()).collect();
      port_names.dedup();
      for port in port_names {
        packets.push(Packet::done(port));
      }

      let invocation = Invocation::new(
        Entity::server("http_client"),
        Entity::operation(route.component.id(), route.operation.operation()),
        None,
        &span,
      );

      let stream = runtime
        .invoke(invocation, packets.into(), None)
        .instrument(span)
        .await
        .map_err(|e| HttpError::OperationError(e.to_string()))?;
      let json = stream_to_json(stream).await?;
      return Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(json.to_string()))
        .map_err(|e| HttpError::OperationError(e.to_string()));
    }
    Ok(
      Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap(),
    )
  }
}

impl Service<Request<Body>> for RestHandler {
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

pub(super) struct RestRoute {
  config: wick_config::config::RestRoute,
  route: route::Route,
  component: ImportBinding,
  operation: ComponentOperationExpression,
}

impl RestRoute {
  pub(super) fn new(
    config: wick_config::config::RestRoute,
    component: ImportBinding,
  ) -> Result<Self, wick_interface_types::ParserError> {
    let route = route::Route::parse(config.uri())?;
    let operation = config.operation().clone();
    Ok(Self {
      config,
      route,
      component,
      operation,
    })
  }
}
