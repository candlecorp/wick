#![allow(clippy::needless_pass_by_value)]

mod component_utils;
mod conversions;
mod proxy_router;
mod raw_router;
mod rest_router;
mod service_factory;

mod static_router;
use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use futures::future::BoxFuture;
use hyper::{Body, Request, Response, Server};
use parking_lot::Mutex;
use serde_json::json;
use structured_output::StructuredOutput;
use tokio::task::JoinHandle;
use tracing::Span;
use wick_config::config::{
  AppConfiguration,
  Codec,
  ImportBinding,
  ProxyRouterConfig,
  RawRouterConfig,
  RestRouterConfig,
  StaticRouterConfig,
  WickRouter,
};
use wick_packet::{Entity, RuntimeConfig};

use self::static_router::StaticRouter;
use super::{resolve_or_import_component, resolve_ref, Trigger, TriggerKind};
use crate::dev::prelude::{RuntimeError, *};
use crate::resources::{Resource, ResourceKind};
use crate::runtime::RuntimeConstraint;
use crate::triggers::build_trigger_runtime;
use crate::triggers::http::proxy_router::ProxyRouter;
use crate::triggers::http::raw_router::RawComponentRouter;
use crate::triggers::http::rest_router::{RestRoute, RestRouter};
use crate::triggers::http::service_factory::ServiceFactory;
use crate::Runtime;

#[derive(Debug, thiserror::Error)]
enum HttpError {
  #[error("Internal error: {:?}",.0)]
  InternalError(InternalError),

  #[error("Operation error: {0}")]
  OperationError(String),

  #[error("Error in stream for '{0}': {1}")]
  OutputStream(String, String),

  #[error("Unsupported HTTP method: {0}")]
  UnsupportedMethod(String),

  #[error("Unsupported HTTP version: {0}")]
  UnsupportedVersion(String),

  #[error("Missing query parameters: {}", .0.join(", "))]
  MissingQueryParameters(Vec<String>),

  #[error("Could not decode body as JSON: {0}")]
  InvalidBody(serde_json::Error),

  #[error("Invalid status code: {0}")]
  InvalidStatusCode(String),

  #[error("Invalid parameter value: {0}")]
  InvalidParameter(String),

  #[error("Could not serialize output into '{0}' codec: {1}")]
  Codec(Codec, String),

  #[error("Could not read output as base64 bytes: {0}")]
  Bytes(String),

  #[error("Invalid header name: {0}")]
  InvalidHeaderName(String),

  #[error("Invalid header value: {0}")]
  InvalidHeaderValue(String),

  #[error("Invalid path or query parameters: {0}")]
  InvalidUri(String),

  #[error("Invalid pre-request middleware response: {0}")]
  InvalidPreRequestResponse(String),

  #[error("Pre-request middleware '{0}' did not provide a request or response")]
  PreRequestResponseNoData(Entity),

  #[error("Post-request middleware '{0}' did not provide a response")]
  PostRequestResponseNoData(Entity),

  #[error("Invalid post-request middleware response: {0}")]
  InvalidPostRequestResponse(String),

  #[error("Error deserializing response on port {0}: {1}")]
  Deserialize(String, String),

  #[error("URI {0} could not be parsed: {1}")]
  RouteSyntax(String, String),
}

#[derive(Debug)]
enum InternalError {
  Builder,
}

#[derive()]
#[must_use]
struct HttpInstance {
  handle: JoinHandle<()>,
  shutdown_tx: tokio::sync::oneshot::Sender<()>,
  running_rx: Option<tokio::sync::oneshot::Receiver<()>>,
  pub(super) addr: SocketAddr,
}

impl HttpInstance {
  async fn new(engine: Runtime, routers: Vec<HttpRouter>, initiating_span: &Span, socket: &SocketAddr) -> Self {
    let span = debug_span!("http_server", %socket);
    span.follows_from(initiating_span);

    span.in_scope(|| trace!(%socket,"http server starting"));
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let (running_tx, running_rx) = tokio::sync::oneshot::channel::<()>();
    let server = Server::bind(socket).serve(ServiceFactory::new(engine, routers, span.clone()));
    let shutdown_span = span.clone();
    let handle = tokio::spawn(async move {
      let _ = server
        .with_graceful_shutdown(async move {
          match rx.await {
            Ok(_) => shutdown_span.in_scope(|| trace!("http server received shutdown signal")),
            Err(_) => shutdown_span.in_scope(|| trace!("http server shutdown signal dropped")),
          }
          shutdown_span.in_scope(|| trace!("http server shutting down"));
        })
        .await;
      let _ = running_tx.send(());
    });
    span.in_scope(|| trace!(%socket,"http server started"));

    Self {
      handle,
      shutdown_tx: tx,
      running_rx: Some(running_rx),
      addr: *socket,
    }
  }

  async fn shutdown(self) -> Result<(), RuntimeError> {
    debug!("shutting down http server");
    self.shutdown_tx.send(()).map_err(|_| {
      RuntimeError::ShutdownFailed(
        TriggerKind::Http,
        "could not send shutdown signal; server may have already died".to_owned(),
      )
    })?;
    self.handle.await.map_err(|_| {
      RuntimeError::ShutdownFailed(
        TriggerKind::Http,
        "waiting for server process to stop after sending shutdown signal failed".to_owned(),
      )
    })?;
    Ok(())
  }
}

#[derive(Debug, Clone)]
enum HttpRouter {
  Raw(RawRouterHandler),
}

impl HttpRouter {
  fn path(&self) -> &str {
    match self {
      HttpRouter::Raw(r) => &r.path,
    }
  }
}

#[derive(Clone)]
struct RawRouterHandler {
  path: String,
  component: Arc<dyn RawRouter + Send + Sync>,
  middleware: RouterMiddleware,
}
impl std::fmt::Debug for RawRouterHandler {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("RawRouterHandler").field("path", &self.path).finish()
  }
}

#[derive(Debug, Clone)]
struct RouterOperation {
  operation: String,
  component: String,
  codec: Codec,
  config: Option<RuntimeConfig>,
  path: String,
}

#[derive(Debug, Clone)]
struct RouterMiddleware {
  request: Vec<(Entity, Option<RuntimeConfig>)>,
  #[allow(unused)]
  response: Vec<(Entity, Option<RuntimeConfig>)>,
}

impl RouterMiddleware {
  pub(crate) fn new(
    request: Vec<(Entity, Option<RuntimeConfig>)>,
    response: Vec<(Entity, Option<RuntimeConfig>)>,
  ) -> Self {
    Self { request, response }
  }
}

#[derive(Default)]
pub(crate) struct Http {
  instance: Arc<Mutex<Option<HttpInstance>>>,
  span: Option<Span>,
}

impl Http {
  pub(crate) fn load() -> Result<Arc<dyn Trigger + Send + Sync>, RuntimeError> {
    Ok(Arc::new(Self::default()))
  }

  async fn handle(
    &self,
    app_config: AppConfiguration,
    config: config::HttpTriggerConfig,
    resources: Arc<HashMap<String, Resource>>,
    span: Span,
    socket: &SocketAddr,
  ) -> Result<HttpInstance, RuntimeError> {
    let mut rt = build_trigger_runtime(&app_config, span.clone())?;

    let routers = span.in_scope(|| {
      let mut routers = Vec::new();
      for (i, router) in config.routers().iter().enumerate() {
        info!(path = router.path(), kind = %router.kind(), "registering http router");

        let (router_bindings, router, constraints) = match router {
          config::HttpRouterConfig::RawRouter(r) => register_raw_router(i, &app_config, r)?,
          config::HttpRouterConfig::StaticRouter(r) => register_static_router(i, resources.clone(), &app_config, r)?,
          config::HttpRouterConfig::ProxyRouter(r) => register_proxy_router(i, resources.clone(), &app_config, r)?,
          config::HttpRouterConfig::RestRouter(r) => register_rest_router(i, resources.clone(), &app_config, r)?,
        };
        for constraint in constraints {
          rt.add_constraint(constraint);
        }
        for binding in router_bindings {
          rt.add_import(binding);
        }

        routers.push(router);
      }
      debug!(?routers, "http routers");
      Ok::<_, RuntimeError>(routers)
    })?;

    let engine = rt.build(None).await?;

    let instance = HttpInstance::new(engine, routers, &span, socket).await;

    Ok(instance)
  }
}

fn index_to_router_id(index: usize) -> String {
  format!("router_{}", index)
}

fn resolve_middleware_components(
  router_index: usize,
  app_config: &AppConfiguration,
  router: &impl WickRouter,
) -> Result<(RouterMiddleware, Vec<ImportBinding>), RuntimeError> {
  let mut request_operations = Vec::new();
  let mut response_operations = Vec::new();
  let mut bindings = Vec::new();
  if let Some(middleware) = router.middleware() {
    for (i, operation) in middleware.request().iter().enumerate() {
      let (name, binding) = resolve_or_import_component(
        app_config,
        format!("{}_request_middleware_{}", router_index, i),
        operation,
      )?;
      if let Some(binding) = binding {
        bindings.push(binding);
      }
      request_operations.push((name, operation.config().and_then(|v| v.value().cloned())));
    }
    for (i, operation) in middleware.response().iter().enumerate() {
      let (name, binding) = resolve_or_import_component(
        app_config,
        format!("{}_request_middleware_{}", router_index, i),
        operation,
      )?;
      if let Some(binding) = binding {
        bindings.push(binding);
      }
      response_operations.push((name, operation.config().and_then(|v| v.value().cloned())));
    }
  }
  let middleware = RouterMiddleware::new(request_operations, response_operations);
  Ok((middleware, bindings))
}

fn register_raw_router(
  index: usize,
  app_config: &AppConfiguration,
  router_config: &RawRouterConfig,
) -> Result<(Vec<ImportBinding>, HttpRouter, Vec<RuntimeConstraint>), RuntimeError> {
  trace!(index, "registering raw router");
  let (middleware, mut bindings) = resolve_middleware_components(index, app_config, router_config)?;

  let router_component = resolve_ref(app_config, router_config.operation().component())?;
  let router_binding = config::ImportBinding::component(index_to_router_id(index), router_component);
  let router = RouterOperation {
    operation: router_config.operation().name().to_owned(),
    component: index_to_router_id(index),
    codec: router_config.codec().copied().unwrap_or_default(),
    config: router_config.operation().config().and_then(|v| v.value().cloned()),
    path: router_config.path().to_owned(),
  };

  let constraint = RuntimeConstraint::Operation {
    entity: Entity::operation(&router.component, &router.operation),
    signature: operation! { "..." => {
        inputs : {
          "request" => "object",
          "body" => "object",
        },
        outputs : {
          "response" => "object",
          "body" => "object",
        },
      }
    },
  };
  let router = RawComponentRouter::new(router);

  bindings.push(router_binding);
  Ok((
    bindings,
    HttpRouter::Raw(RawRouterHandler {
      path: router_config.path().to_owned(),
      component: Arc::new(router),
      middleware,
    }),
    vec![constraint],
  ))
}

fn register_static_router(
  index: usize,
  resources: Arc<HashMap<String, Resource>>,
  app_config: &AppConfiguration,
  router_config: &StaticRouterConfig,
) -> Result<(Vec<ImportBinding>, HttpRouter, Vec<RuntimeConstraint>), RuntimeError> {
  trace!(index, "registering static router");
  let (middleware, mut bindings) = resolve_middleware_components(index, app_config, router_config)?;
  let volume = resources.get(router_config.volume()).ok_or_else(|| {
    RuntimeError::ResourceNotFound(
      TriggerKind::Http,
      format!("volume {} not found", router_config.volume()),
    )
  })?;
  let volume = match volume {
    Resource::Volume(s) => s.clone(),
    _ => {
      return Err(RuntimeError::InvalidResourceType(
        TriggerKind::Http,
        ResourceKind::Volume,
        volume.kind(),
      ))
    }
  };

  let fallback = router_config.fallback().cloned();

  let router = StaticRouter::new(volume, Some(router_config.path().to_owned()), fallback);
  let router_component = config::ComponentDefinition::Native(config::components::NativeComponent {});
  let router_binding = config::ImportBinding::component(index_to_router_id(index), router_component);
  bindings.push(router_binding);
  Ok((
    bindings,
    HttpRouter::Raw(RawRouterHandler {
      path: router_config.path().to_owned(),
      component: Arc::new(router),
      middleware,
    }),
    vec![],
  ))
}

fn register_rest_router(
  index: usize,
  _resources: Arc<HashMap<String, Resource>>,
  app_config: &AppConfiguration,
  router_config: &RestRouterConfig,
) -> Result<(Vec<ImportBinding>, HttpRouter, Vec<RuntimeConstraint>), RuntimeError> {
  trace!(index, "registering rest router");
  let (middleware, mut bindings) = resolve_middleware_components(index, app_config, router_config)?;
  let mut routes = Vec::new();
  for (i, route) in router_config.routes().iter().enumerate() {
    info!(sub_path = route.sub_path(), "registering rest route");

    let route_component = resolve_ref(app_config, route.operation().component())?;
    let route_binding =
      config::ImportBinding::component(format!("{}_{}", index_to_router_id(index), i), route_component);
    let config = route_binding.config().cloned();
    let route = RestRoute::new(route.clone(), route_binding.clone(), config).map_err(|e| {
      RuntimeError::InitializationFailed(format!(
        "could not intitialize rest router for route {}: {}",
        route.sub_path(),
        e
      ))
    })?;
    bindings.push(route_binding);
    routes.push(route);
  }

  let router = RestRouter::new(app_config, router_config.clone(), routes)
    .map_err(|e| RuntimeError::InitializationFailed(e.to_string()))?;
  let router_component = config::ComponentDefinition::Native(config::components::NativeComponent {});
  let router_binding = config::ImportBinding::component(index_to_router_id(index), router_component);
  bindings.push(router_binding);
  Ok((
    bindings,
    HttpRouter::Raw(RawRouterHandler {
      path: router_config.path().to_owned(),
      component: Arc::new(router),
      middleware,
    }),
    vec![],
  ))
}

fn register_proxy_router(
  index: usize,
  resources: Arc<HashMap<String, Resource>>,
  app_config: &AppConfiguration,
  router_config: &ProxyRouterConfig,
) -> Result<(Vec<ImportBinding>, HttpRouter, Vec<RuntimeConstraint>), RuntimeError> {
  trace!(index, "registering proxy router");
  let (middleware, mut bindings) = resolve_middleware_components(index, app_config, router_config)?;
  let url = resources.get(router_config.url()).ok_or_else(|| {
    RuntimeError::ResourceNotFound(
      TriggerKind::Http,
      format!("url resource {} not found", router_config.url()),
    )
  })?;
  let url = match url {
    Resource::Url(s) => s.clone(),
    _ => {
      return Err(RuntimeError::InvalidResourceType(
        TriggerKind::Http,
        ResourceKind::Url,
        url.kind(),
      ))
    }
  };
  let strip_path = if router_config.strip_path() {
    Some(router_config.path().to_owned())
  } else {
    None
  };
  let router = ProxyRouter::new(url, strip_path);
  let router_component = config::ComponentDefinition::Native(config::components::NativeComponent {});
  let router_binding = config::ImportBinding::component(index_to_router_id(index), router_component);
  bindings.push(router_binding);
  Ok((
    bindings,
    HttpRouter::Raw(RawRouterHandler {
      path: router_config.path().to_owned(),
      component: Arc::new(router),
      middleware,
    }),
    vec![],
  ))
}

#[async_trait]
impl Trigger for Http {
  async fn run(
    &self,
    _name: String,
    app_config: AppConfiguration,
    config: config::TriggerDefinition,
    resources: Arc<HashMap<String, Resource>>,
    span: Span,
  ) -> Result<StructuredOutput, RuntimeError> {
    span.in_scope(|| debug!(kind = %TriggerKind::Http, "trigger:run"));
    let config = if let config::TriggerDefinition::Http(config) = config {
      config
    } else {
      return Err(RuntimeError::InvalidTriggerConfig(TriggerKind::Http));
    };
    let resource_name = config.resource();
    let resource = resources
      .get(resource_name)
      .ok_or_else(|| RuntimeError::ResourceNotFound(TriggerKind::Http, resource_name.to_owned()))?;
    let socket = match resource {
      Resource::TcpPort(s) => *s,
      _ => {
        return Err(RuntimeError::InvalidResourceType(
          TriggerKind::Http,
          ResourceKind::TcpPort,
          resource.kind(),
        ))
      }
    };

    let instance = self
      .handle(app_config, config, resources, span.clone(), &socket)
      .await?;

    let output = StructuredOutput::new(
      format!("HTTP Server started on {}", instance.addr),
      json!({"ip": instance.addr.ip(),"port": instance.addr.port()}),
    );

    span.in_scope(|| info!(address=%instance.addr,"http trigger started"));

    self.instance.lock().replace(instance);

    Ok(output)
  }

  async fn shutdown_gracefully(self) -> Result<(), RuntimeError> {
    self
      .span
      .clone()
      .unwrap_or_else(Span::current)
      .in_scope(|| info!("HTTP server shutting down gracefully"));
    if self.instance.lock().is_none() {
      return Ok(());
    }
    let instance = self.instance.lock().take().unwrap();
    instance.shutdown().await?;
    Ok(())
  }

  async fn wait_for_done(&self) {
    let rx = if let Some(instance) = self.instance.lock().as_mut() {
      instance.running_rx.take()
    } else {
      None
    };
    if let Some(rx) = rx {
      let _ = rx.await;
    } else {
      error!("http trigger not running");
    }
  }
}

impl fmt::Display for Http {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Cli Trigger",)
  }
}

trait RawRouter {
  fn handle(
    &self,
    remote_addr: SocketAddr,
    runtime: Arc<Runtime>,
    request: Request<Body>,
  ) -> BoxFuture<Result<Response<Body>, HttpError>>;
}

#[cfg(test)]
mod test {

  // "port_limited" tests are grouped together and run on a single thread to prevent port contention
  mod port_limited {

    use anyhow::Result;

    use super::super::*;
    use crate::test::{load_example, load_test_manifest};

    static PORT: &str = "9005";

    async fn get(path: &str) -> Result<reqwest::Response> {
      let client = reqwest::Client::new();
      let res = client.get(format!("http://0.0.0.0:{}{}", PORT, path)).send().await?;
      Ok(res)
    }

    #[test_logger::test(tokio::test)]
    async fn test_raw_router() -> Result<()> {
      std::env::set_var("HTTP_PORT", PORT);
      let app_config = load_test_manifest("app_config/app-http-server-wasm.wick")
        .await?
        .try_app_config()?;

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
        .post(format!("http://0.0.0.0:{}", PORT))
        .body(r#"{"message": "my json message"}"#)
        .send()
        .await?
        .text()
        .await?;

      println!("{:#?}", res);
      assert_eq!(res, r#"{"output_message":"egassem nosj ym"}"#);
      trigger.shutdown_gracefully().await?;

      Ok(())
    }

    #[test_logger::test(tokio::test)]
    async fn test_middleware() -> Result<()> {
      std::env::set_var("HTTP_PORT", PORT);
      let app_config = load_example("http/middleware.wick").await?.try_app_config()?;

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
      // requests to /redirect should result in a redirected response.
      let res = get("/redirect?url=https://google.com/").await?.text().await?;

      println!("{:#?}", res);

      assert!(res.contains("Google"));

      // requests to /google should result in a redirected response (from a composite component)
      let res = get("/google").await?.text().await?;

      println!("{:#?}", res);

      assert!(res.contains("Google"));

      // check that other requests still go through.
      let res = get("/this/FIRST_VALUE/some/222?third=third_a&fourth=true").await?;

      // check that our request middleware modified a request header & that it made its way to the response middleware
      let header = res.headers().get("x-wick-redirect").unwrap();
      assert_eq!(header, "false");
      // check our response middleware added a header.
      let header = res.headers().get("x-wick-count").unwrap();
      assert_eq!(header, "3");
      let res: serde_json::Value = res.json().await?;
      println!("{:#?}", res);
      assert_eq!(
        res,
        json!({"first":"FIRST_VALUE", "second": 222,"third":"third_a", "fourth":true })
      );

      // check that our response middleware has been called again.
      let res = get("/this/FIRST_VALUE/some/222?third=third_a&fourth=true").await?;
      let header = res.headers().get("x-wick-count").unwrap();
      assert_eq!(header, "4");

      trigger.shutdown_gracefully().await?;

      Ok(())
    }

    #[test_logger::test(tokio::test)]
    async fn test_rest_router() -> Result<()> {
      std::env::set_var("HTTP_PORT", PORT);
      let app_config = load_example("http/rest-router.wick").await?.try_app_config()?;

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

      let res: serde_json::Value = get("/this/FIRST_VALUE/some/222?third=third_a&fourth=true")
        .await?
        .json()
        .await?;

      println!("{:#?}", res);
      assert_eq!(
        res,
        json!({"first":"FIRST_VALUE", "second": 222,"third":["third_a"], "fourth":true })
      );

      let res: serde_json::Value = get("/this/FIRST_VALUE/some/222?third=third_a&third=third_b&fourth=true")
        .await?
        .json()
        .await?;

      println!("{:#?}", res);
      assert_eq!(
        res,
        json!({"first":"FIRST_VALUE", "second": 222,"third":["third_a","third_b"], "fourth":true })
      );
      trigger.shutdown_gracefully().await?;

      Ok(())
    }
  }
}
