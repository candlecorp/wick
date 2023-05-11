use std::net::SocketAddr;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::future::BoxFuture;
use hyper::service::Service;
use hyper::{Body, Request, Response, StatusCode};
use wick_config::config::{ComponentOperationExpression, ImportBinding, RestRouterConfig};
use wick_packet::{Entity, Invocation, Packet};
mod route;

use super::{HttpError, RawRouter};
use crate::triggers::http::component_utils::stream_to_json;
use crate::Runtime;

static ID: &str = "wick:http:rest";

#[derive()]
#[must_use]
pub(super) struct RestRouter {
  routes: Arc<Vec<RestRoute>>,
}

impl RestRouter {
  pub(super) fn new(config: RestRouterConfig, routes: Vec<RestRoute>) -> Self {
    let title = config
      .info()
      .and_then(|i| i.title().cloned())
      .unwrap_or_else(|| "Untitled API".to_owned());
    debug!(api = %title, "{}: serving", ID);
    let routes = Arc::new(routes);

    Self { routes }
  }
}

impl RawRouter for RestRouter {
  fn handle(
    &self,
    _remote_addr: SocketAddr,
    runtime: Arc<Runtime>,
    request: Request<Body>,
  ) -> BoxFuture<Result<Response<Body>, HttpError>> {
    let handler = RestHandler::new(self.routes.clone(), runtime);
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
}

impl RestHandler {
  fn new(routes: Arc<Vec<RestRoute>>, runtime: Arc<Runtime>) -> Self {
    RestHandler { runtime, routes }
  }

  /// Serve a request.
  #[allow(clippy::unused_async)]
  async fn serve(self, request: Request<Body>) -> Result<Response<Body>, HttpError> {
    let Self { routes, runtime, .. } = self;

    for route in routes.iter() {
      if !route.config.methods().is_empty() && !route.config.methods().contains(&request.method().to_string()) {
        continue;
      }
      let Some((path_params, query_params)) = route.route.compare(request.uri().path(), request.uri().query()) else {
        continue
      };
      let mut packets: Vec<_> = path_params
        .iter()
        .chain(query_params.iter())
        .map(|f| Packet::encode(f.name(), f.value()))
        .collect();
      let mut port_names: Vec<_> = packets.iter().map(|p| p.port().to_owned()).collect();
      port_names.dedup();
      for port in port_names {
        packets.push(Packet::done(port));
      }
      let invocation = Invocation::new(
        Entity::server("http_client"),
        Entity::operation(route.component.id(), route.operation.operation()),
        None,
      );
      let stream = runtime
        .invoke(invocation, packets.into(), None)
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
