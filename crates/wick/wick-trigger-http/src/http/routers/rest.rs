use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::task::Poll;

use hyper::body::to_bytes;
use hyper::service::Service;
use hyper::{Body, Request, Response, StatusCode};
use tracing::{Instrument, Span};
use uuid::Uuid;
use wick_config::config::{
  AppConfiguration,
  BoundIdentifier,
  ComponentOperationExpression,
  HttpMethod,
  RestRouterConfig,
  WickRouter,
};
use wick_packet::{Entity, InherentData, Invocation, Packet, PacketExt};
mod error;
mod openapi;
mod route;

use wick_runtime::Runtime;
use wick_trigger::resources::Resource;

use self::error::RestError;
use crate::http::component_utils::stream_to_json;
use crate::http::middleware::resolve_middleware_components;
use crate::http::{BoxFuture, HttpError, HttpRouter, RawRouter, RawRouterHandler};

pub(crate) const OPENAPI_PATH: &str = "/openapi.json";

#[derive()]
#[must_use]
pub(super) struct RestRouter {
  context: Arc<Context>,
}

impl RestRouter {
  pub(super) fn new(
    app_config: &AppConfiguration,
    config: RestRouterConfig,
    routes: Vec<RestRoute>,
  ) -> Result<Self, RestError> {
    let title = config
      .info()
      .and_then(|i| i.title().cloned())
      .unwrap_or_else(|| "Untitled API".to_owned());

    debug!(api = %title, path=%config.path(), "router:rest:serving");
    for route in &routes {
      debug!(route = ?route.route, "router:rest:route");
    }

    let oapi = config.tools().map_or(false, |t| t.openapi());
    let oapi = oapi
      .then(|| {
        info!(
          path = format!("{}{}", config.path(), OPENAPI_PATH),
          "openapi schema enabled"
        );
        openapi::generate_openapi(app_config, &config, &routes)
      })
      .transpose()?;

    Ok(Self {
      context: Arc::new(Context {
        routes,
        root: config.path().to_owned(),
        openapi: oapi,
      }),
    })
  }
}

impl RawRouter for RestRouter {
  fn handle(
    &self,
    tx_id: Uuid,
    _remote_addr: SocketAddr,
    runtime: Runtime,
    request: Request<Body>,
    span: &Span,
  ) -> BoxFuture<Result<Response<Body>, HttpError>> {
    let span = info_span!(parent: span, "rest");

    let handler = RestHandler::new(tx_id, runtime, self.context.clone(), span);
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

struct Context {
  root: String,
  routes: Vec<RestRoute>,
  openapi: Option<openapiv3::OpenAPI>,
}

#[derive(Clone)]
struct RestHandler {
  context: Arc<Context>,
  runtime: Runtime,
  tx_id: Uuid,
  span: Span,
}

impl RestHandler {
  fn new(tx_id: Uuid, runtime: Runtime, context: Arc<Context>, span: Span) -> Self {
    RestHandler {
      tx_id,
      context,
      runtime,
      span,
    }
  }

  /// Serve a request.
  #[allow(clippy::too_many_lines)]
  async fn serve(self, request: Request<Body>) -> Result<Response<Body>, HttpError> {
    let Self {
      context,
      runtime,
      span,
      tx_id,
    } = self;
    let method = match *request.method() {
      hyper::Method::GET => wick_config::config::HttpMethod::Get,
      hyper::Method::POST => wick_config::config::HttpMethod::Post,
      hyper::Method::PUT => wick_config::config::HttpMethod::Put,
      hyper::Method::DELETE => wick_config::config::HttpMethod::Delete,
      _ => {
        return Ok(
          Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
        )
      }
    };

    let path = request.uri().path().trim_start_matches(context.root.as_str());
    if let Some(openapi) = context.openapi.as_ref() {
      if path == OPENAPI_PATH {
        span.in_scope(|| debug!("serving openapi schema"));
        let json = serde_json::to_string(openapi).unwrap();
        return Ok(
          Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::from(json))
            .unwrap(),
        );
      }
    }

    for route in &context.routes {
      if !route.config.methods().is_empty() && !route.config.methods().contains(&method) {
        continue;
      }
      let Some((path_params, query_params)) = route.route.compare(path, request.uri().query())? else {
        continue;
      };
      let uri = request.uri().clone();
      span.in_scope(
        || trace!(route = %uri, path_params=?path_params, query_params=?query_params, "incoming http request"),
      );
      let mut packets: Vec<_> = path_params
        .iter()
        .chain(query_params.iter())
        .map(|f| Packet::encode(f.name(), f.value()))
        .collect();

      let (_, body) = request.into_parts();

      let body_bytes = to_bytes(body).await.unwrap_or_default();
      let body = String::from_utf8_lossy(&body_bytes);

      span.in_scope(|| trace!(route = %uri, len=body_bytes.len(), "body"));

      if !matches!(method, HttpMethod::Get) {
        let payload: Option<serde_json::Value> = if body.trim().is_empty() {
          None
        } else {
          Some(serde_json::from_str(&body).map_err(HttpError::InvalidBody)?)
        };

        packets.push(Packet::encode("input", payload));
      }

      let mut port_names: Vec<_> = packets.iter().map(|p| p.port().to_owned()).collect();
      port_names.dedup();
      for port in port_names {
        packets.push(Packet::done(port));
      }

      let invocation = Invocation::new_with_id(
        tx_id,
        Entity::server("http"),
        Entity::operation(&route.component, route.operation.name()),
        packets,
        InherentData::unsafe_default(),
        &span,
      );
      let runtime_config = route.operation.config().and_then(|c| c.value().cloned());

      let stream = runtime
        .invoke(invocation, runtime_config)
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

  fn poll_ready(&mut self, _cx: &mut std::task::Context) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, request: Request<Body>) -> Self::Future {
    Box::pin(self.clone().serve(request))
  }
}

pub(crate) struct RestRoute {
  config: wick_config::config::RestRoute,
  route: route::Route,
  component: String,
  operation: ComponentOperationExpression,
}

impl RestRoute {
  pub(super) fn new(config: wick_config::config::RestRoute, component_id: String) -> Result<Self, HttpError> {
    let route = route::Route::parse(config.sub_path())
      .map_err(|e| HttpError::RouteSyntax(e.to_string(), config.sub_path().to_owned()))?;
    let operation = config.operation().clone();

    Ok(Self {
      config,
      route,
      component: component_id,
      operation,
    })
  }
}

#[cfg(test)]
mod test {

  // "port_limited" tests are grouped together and run on a single thread to prevent port contention
  mod port_limited {

    use anyhow::Result;
    use wick_trigger::resources::Resource;
    use wick_trigger::{build_trigger_runtime, Trigger};

    use super::super::*;
    use crate::http::Http;
    use crate::test::load_test_manifest;

    static PORT: &str = "9005";

    #[test_logger::test(tokio::test)]
    async fn rest_errors() -> Result<()> {
      std::env::set_var("HTTP_PORT", PORT);
      let app_config = load_test_manifest("app_config/rest-router-errors.wick")
        .await?
        .try_app_config()?;
      let rt = build_trigger_runtime(&app_config, Span::current())?.build(None).await?;

      let trigger = Http::default();
      let resource = Resource::new(app_config.resources().get(0).as_ref().unwrap().kind().clone())?;
      let resources = Arc::new([("http".into(), resource)].iter().cloned().collect());
      let trigger_config = app_config.triggers()[0].clone();
      trigger
        .run(
          "test".to_owned(),
          rt,
          app_config,
          trigger_config,
          resources,
          Span::current(),
        )
        .await?;

      let client = reqwest::Client::new();
      let res = client
        .post(format!("http://0.0.0.0:{}/bad_op", PORT))
        .body(r#"{"message": "my json message"}"#)
        .send()
        .await?;

      assert!(res.status() != 404);
      let body = res.text().await?;
      println!("Response body: \"{}\"", body);
      assert!(body.contains("Internal Server Error"));

      trigger.shutdown_gracefully().await?;
      Ok(())
    }
  }
}

pub(crate) fn register_rest_router(
  index: usize,
  _resources: Arc<HashMap<BoundIdentifier, Resource>>,
  app_config: &AppConfiguration,
  router_config: &RestRouterConfig,
) -> Result<HttpRouter, HttpError> {
  trace!(index, "registering rest router");
  let middleware = resolve_middleware_components(router_config)?;
  let mut routes = Vec::new();

  for route in router_config.routes().iter() {
    info!(sub_path = route.sub_path(), "registering rest route");

    let component_id = route.operation().component_id()?;
    let route = RestRoute::new(route.clone(), component_id.to_owned()).map_err(|e| {
      HttpError::InitializationFailed(format!(
        "could not intitialize rest router for route {}: {}",
        route.sub_path(),
        e
      ))
    })?;
    routes.push(route);
  }

  let router = RestRouter::new(app_config, router_config.clone(), routes)
    .map_err(|e| HttpError::InitializationFailed(e.to_string()))?;
  Ok(HttpRouter::Raw(RawRouterHandler {
    path: router_config.path().to_owned(),
    component: Arc::new(router),
    middleware,
  }))
}
