use std::net::SocketAddr;
use std::sync::Arc;
use std::task::Poll;

use futures::future::BoxFuture;
use hyper::body::to_bytes;
use hyper::service::Service;
use hyper::{Body, Request, Response, StatusCode};
use tracing::{Instrument, Span};
use wick_config::config::{
  AppConfiguration,
  ComponentOperationExpression,
  HttpMethod,
  ImportBinding,
  RestRouterConfig,
  WickRouter,
};
use wick_packet::{Entity, InherentData, Invocation, Packet, RuntimeConfig};
mod error;
mod openapi;
mod route;

use self::error::RestError;
use super::{HttpError, RawRouter};
use crate::triggers::http::component_utils::stream_to_json;
use crate::Runtime;

pub(crate) const OPENAPI_PATH: &str = "openapi.json";

#[derive()]
#[must_use]
pub(super) struct RestRouter {
  span: Span,
  context: Arc<Context>,
}

impl RestRouter {
  pub(super) fn new(
    app_config: &AppConfiguration,
    config: RestRouterConfig,
    routes: Vec<RestRoute>,
  ) -> Result<Self, RestError> {
    let span = debug_span!("http:rest", path = %config.path());

    let title = config
      .info()
      .and_then(|i| i.title().cloned())
      .unwrap_or_else(|| "Untitled API".to_owned());

    span.in_scope(|| {
      debug!(api = %title, path=%config.path(), "serving");
      for route in &routes {
        debug!(route = ?route.route, "route");
      }
    });

    let oapi = config.tools().map_or(false, |t| t.openapi());
    let oapi = if oapi {
      info!(
        path = format!("{}{}", config.path(), OPENAPI_PATH),
        "openapi schema enabled"
      );
      Some(openapi::generate_openapi(app_config, &config, &routes)?)
    } else {
      None
    };

    Ok(Self {
      context: Arc::new(Context {
        routes,
        root: config.path().to_owned(),
        openapi: oapi,
      }),
      span,
    })
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
    let handler = RestHandler::new(runtime, self.context.clone(), span);
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
  runtime: Arc<Runtime>,
  span: Span,
}

impl RestHandler {
  fn new(runtime: Arc<Runtime>, context: Arc<Context>, span: Span) -> Self {
    RestHandler { context, runtime, span }
  }

  /// Serve a request.
  #[allow(clippy::unused_async)]
  async fn serve(self, request: Request<Body>) -> Result<Response<Body>, HttpError> {
    let Self { context, runtime, span } = self;
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

    for route in context.routes.iter() {
      if !route.config.methods().is_empty() && !route.config.methods().contains(&method) {
        continue;
      }
      let Some((path_params, query_params)) = route.route.compare(path, request.uri().query())? else {
        continue
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
        let payload: serde_json::Value = if body.trim().is_empty() {
          serde_json::Value::Null
        } else {
          serde_json::from_str(&body).map_err(HttpError::InvalidBody)?
        };

        packets.push(Packet::encode("input", payload));
      }

      let mut port_names: Vec<_> = packets.iter().map(|p| p.port().to_owned()).collect();
      port_names.dedup();
      for port in port_names {
        packets.push(Packet::done(port));
      }

      let invocation = Invocation::new(
        Entity::server("http"),
        Entity::operation(route.component.id(), route.operation.name()),
        packets,
        InherentData::unsafe_default(),
        &span,
      );

      let stream = runtime
        .invoke(invocation, route.runtime_config.clone())
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
  component: ImportBinding,
  operation: ComponentOperationExpression,
  runtime_config: Option<RuntimeConfig>,
}

impl RestRoute {
  pub(super) fn new(
    config: wick_config::config::RestRoute,
    component: ImportBinding,
    runtime_config: Option<RuntimeConfig>,
  ) -> Result<Self, HttpError> {
    let route = route::Route::parse(config.sub_path())
      .map_err(|e| HttpError::RouteSyntax(e.to_string(), config.sub_path().to_owned()))?;
    let operation = config.operation().clone();

    Ok(Self {
      config,
      route,
      component,
      operation,
      runtime_config,
    })
  }
}

#[cfg(test)]
mod test {

  // "port_limited" tests are grouped together and run on a single thread to prevent port contention
  mod port_limited {

    use anyhow::Result;

    use super::super::*;
    use crate::resources::Resource;
    use crate::test::load_test_manifest;
    use crate::triggers::http::Http;
    use crate::Trigger;

    static PORT: &str = "9005";

    #[test_logger::test(tokio::test)]
    async fn rest_errors() -> Result<()> {
      std::env::set_var("HTTP_PORT", PORT);
      let app_config = load_test_manifest("rest-router-errors.wick").await?.try_app_config()?;

      let trigger = Http::default();
      let resource = Resource::new(app_config.resources().get("http").as_ref().unwrap().kind().clone())?;
      let resources = Arc::new([("http".to_owned(), resource)].iter().cloned().collect());
      let trigger_config = app_config.triggers()[0].clone();
      trigger
        .run(
          "test".to_owned(),
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
