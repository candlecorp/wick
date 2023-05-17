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
use wick_config::config::components::Codec;
use wick_config::config::{
  AppConfiguration,
  ImportBinding,
  ProxyRouterConfig,
  RawRouterConfig,
  RestRouterConfig,
  StaticRouterConfig,
};
use wick_packet::Entity;

use self::static_router::StaticRouter;
use super::{resolve_ref, Trigger, TriggerKind};
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
  #[error("Operation error: {0}")]
  OperationError(String),
  #[error("Unsupported HTTP method: {0}")]
  UnsupportedMethod(String),
  #[error("Unsupported HTTP version: {0}")]
  UnsupportedVersion(String),
  #[error("Missing query parameters: {}", .0.join(", "))]
  MissingQueryParameters(Vec<String>),
  #[error("Invalid status code: {0}")]
  InvalidStatusCode(String),
  #[error("Invalid parameter value: {0}")]
  InvalidParameter(String),
  #[error("Invalid response: {0}")]
  InvalidResponse(String),
  #[error("Error deserializing response on port {0}: {1}")]
  Deserialize(String, String),
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
  async fn new(engine: Runtime, routers: Vec<HttpRouter>, socket: &SocketAddr) -> Self {
    trace!(%socket,"http server starting");
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let (running_tx, running_rx) = tokio::sync::oneshot::channel::<()>();

    let server = Server::bind(socket).serve(ServiceFactory::new(engine, routers));
    let handle = tokio::spawn(async move {
      let _ = server
        .with_graceful_shutdown(async move {
          match rx.await {
            Ok(_) => trace!("http server received shutdown signal"),
            Err(_) => trace!("http server shutdown signal dropped"),
          }
          trace!("http server shutting down");
        })
        .await;
      let _ = running_tx.send(());
    });

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
}

#[derive(Default)]
pub(crate) struct Http {
  instance: Arc<Mutex<Option<HttpInstance>>>,
}

impl Http {
  pub(crate) fn load() -> Result<Arc<dyn Trigger + Send + Sync>, RuntimeError> {
    Ok(Arc::new(Http::load_impl()?))
  }

  pub(crate) fn load_impl() -> Result<Http, RuntimeError> {
    Ok(Self::default())
  }

  async fn handle(
    &self,
    app_config: AppConfiguration,
    config: config::HttpTriggerConfig,
    resources: Arc<HashMap<String, Resource>>,
    socket: &SocketAddr,
  ) -> Result<HttpInstance, RuntimeError> {
    let mut rt = build_trigger_runtime(&app_config)?;
    let mut routers = Vec::new();

    for (i, router) in config.routers().iter().enumerate() {
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
    let engine = rt.build().await?;

    let instance = HttpInstance::new(engine, routers, socket).await;

    Ok(instance)
  }
}

fn index_to_router_id(index: usize) -> String {
  format!("router_{}", index)
}

fn register_raw_router(
  index: usize,
  app_config: &AppConfiguration,
  router_config: &RawRouterConfig,
) -> Result<(Vec<ImportBinding>, HttpRouter, Vec<RuntimeConstraint>), RuntimeError> {
  trace!(index, "registering raw router");
  let router_component = resolve_ref(app_config, router_config.operation().component())?;
  let router_binding = config::ImportBinding::component(index_to_router_id(index), router_component);
  let router = RouterOperation {
    operation: router_config.operation().operation().to_owned(),
    component: index_to_router_id(index),
    codec: router_config.codec().copied().unwrap_or_default(),
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

  Ok((
    vec![router_binding],
    HttpRouter::Raw(RawRouterHandler {
      path: router_config.path().to_owned(),
      component: Arc::new(router),
    }),
    vec![constraint],
  ))
}

fn register_static_router(
  index: usize,
  resources: Arc<HashMap<String, Resource>>,
  _app_config: &AppConfiguration,
  router_config: &StaticRouterConfig,
) -> Result<(Vec<ImportBinding>, HttpRouter, Vec<RuntimeConstraint>), RuntimeError> {
  trace!(index, "registering static router");
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
  let router = StaticRouter::new(volume, Some(router_config.path().to_owned()));
  let router_component = config::ComponentDefinition::Native(config::components::NativeComponent {});
  let router_binding = config::ImportBinding::component(index_to_router_id(index), router_component);
  Ok((
    vec![router_binding],
    HttpRouter::Raw(RawRouterHandler {
      path: router_config.path().to_owned(),
      component: Arc::new(router),
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
  let mut bindings = Vec::new();
  let mut routes = Vec::new();
  for (i, route) in router_config.routes().iter().enumerate() {
    let route_component = resolve_ref(app_config, route.operation().component())?;
    let route_binding =
      config::ImportBinding::component(format!("{}_{}", index_to_router_id(index), i), route_component);
    let route = RestRoute::new(route.clone(), route_binding.clone()).map_err(|e| {
      RuntimeError::InitializationFailed(format!(
        "could not intitialize rest router for route {}: {}",
        route.uri(),
        e
      ))
    })?;
    bindings.push(route_binding);
    routes.push(route);
  }

  let router = RestRouter::new(router_config.clone(), routes);
  let router_component = config::ComponentDefinition::Native(config::components::NativeComponent {});
  let router_binding = config::ImportBinding::component(index_to_router_id(index), router_component);
  bindings.push(router_binding);
  Ok((
    bindings,
    HttpRouter::Raw(RawRouterHandler {
      path: router_config.path().to_owned(),
      component: Arc::new(router),
    }),
    vec![],
  ))
}

fn register_proxy_router(
  index: usize,
  resources: Arc<HashMap<String, Resource>>,
  _app_config: &AppConfiguration,
  router_config: &ProxyRouterConfig,
) -> Result<(Vec<ImportBinding>, HttpRouter, Vec<RuntimeConstraint>), RuntimeError> {
  trace!(index, "registering proxy router");
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
  Ok((
    vec![router_binding],
    HttpRouter::Raw(RawRouterHandler {
      path: router_config.path().to_owned(),
      component: Arc::new(router),
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
  ) -> Result<StructuredOutput, RuntimeError> {
    debug!(kind = %TriggerKind::Http, "trigger:run");
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

    let instance = self.handle(app_config, config, resources, &socket).await?;
    let output = StructuredOutput::new(
      format!("HTTP Server started on {}", instance.addr),
      json!({"ip": instance.addr.ip(),"port": instance.addr.port()}),
    );
    info!("{}", output.lines());

    self.instance.lock().replace(instance);

    Ok(output)
  }

  async fn shutdown_gracefully(self) -> Result<(), RuntimeError> {
    info!("HTTP server shutting down gracefully");
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
  use std::path::PathBuf;

  use anyhow::Result;

  use super::*;

  #[test_logger::test(tokio::test)]
  async fn test_raw_router() -> Result<()> {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest_dir = crate_dir.join("../../../tests/testdata/manifests");
    let yaml = manifest_dir.join("app-http-server-wasm.wick");

    let app_config = config::WickConfiguration::fetch(yaml.to_string_lossy(), Default::default())
      .await?
      .try_app_config()?;

    let trigger = Http::load_impl()?;
    let resource = Resource::new(app_config.resources().get("http").as_ref().unwrap().kind().clone())?;
    let resources = Arc::new([("http".to_owned(), resource)].iter().cloned().collect());
    let trigger_config = app_config.triggers()[0].clone();
    trigger
      .run("test".to_owned(), app_config, trigger_config, resources)
      .await?;
    let client = reqwest::Client::new();
    let res = client
      .post("http://0.0.0.0:8999")
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
  async fn test_rest_router() -> Result<()> {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest_dir = crate_dir.join("../../../examples/");

    let yaml = manifest_dir.join("http-restapi.wick");
    let app_config = config::WickConfiguration::fetch(yaml.to_string_lossy(), Default::default())
      .await?
      .try_app_config()?;
    let trigger = Http::load_impl()?;
    let resource = Resource::new(app_config.resources().get("http").as_ref().unwrap().kind().clone())?;
    let resources = Arc::new([("http".to_owned(), resource)].iter().cloned().collect());
    let trigger_config = app_config.triggers()[0].clone();
    trigger
      .run("test".to_owned(), app_config, trigger_config, resources)
      .await?;
    let client = reqwest::Client::new();
    let res: serde_json::Value = client
      .get("http://0.0.0.0:8999/this/FIRST_VALUE/some/222?third=third_a&fourth=true")
      .send()
      .await?
      .json()
      .await?;

    println!("{:#?}", res);
    assert_eq!(
      res,
      json!({"first":"FIRST_VALUE", "second": 222,"third":"third_a", "fourth":true })
    );
    trigger.shutdown_gracefully().await?;
    // Todo, fix parsing query string list parameters
    // let res: serde_json::Value = client
    //   .get("http://0.0.0.0:8999/this/FIRST_VALUE/some/222?third=third_a&third=third_b&fourth=true")
    //   .send()
    //   .await?
    //   .json()
    //   .await?;

    // println!("{:#?}", res);
    // assert_eq!(
    //   res,
    //   json!({"first":"FIRST_VALUE", "second": 222,"third":["third_a","third_b"], "fourth":true })
    // );
    // trigger.shutdown_gracefully().await?;

    Ok(())
  }
}
