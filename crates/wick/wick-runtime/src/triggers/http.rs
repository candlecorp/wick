#![allow(clippy::needless_pass_by_value)]

mod conversions;
mod service_factory;
use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use hyper::Server;
use parking_lot::Mutex;
use tokio::task::JoinHandle;
use wick_config::{AppConfiguration, BoundComponent, HttpRouterConfig, HttpTriggerConfig, TriggerDefinition};

use super::{resolve_ref, Trigger, TriggerKind};
use crate::dev::prelude::RuntimeError;
use crate::resources::{Resource, ResourceKind};
use crate::triggers::http::service_factory::ServiceFactory;
use crate::Network;

#[derive(Debug, thiserror::Error)]
enum HttpError {
  #[error("Unsupported HTTP method: {0}")]
  UnsupportedMethod(String),
  #[error("Unsupported HTTP version: {0}")]
  UnsupportedVersion(String),
  #[error("Invalid status code: {0}")]
  InvalidStatusCode(String),
  #[error("Not found: {0}")]
  NotFound(String),
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
}

impl HttpInstance {
  async fn new(network: Network, routers: Vec<HttpRouter>, socket: &SocketAddr) -> Self {
    trace!(%socket,"http server starting");
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    let server = Server::bind(socket).serve(ServiceFactory::new(network, routers));
    let handle = tokio::spawn(async move {
      let _ = server
        .with_graceful_shutdown(async move {
          let _ = rx.await;
          trace!("http server shutting down");
        })
        .await;
    });

    Self {
      handle,
      shutdown_tx: tx,
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
struct HttpRouter {
  path: String,
  operation: String,
  component: String,
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

  async fn handle_command(
    &self,
    app_config: AppConfiguration,
    config: HttpTriggerConfig,
    socket: &SocketAddr,
  ) -> Result<HttpInstance, RuntimeError> {
    let mut network = crate::NetworkBuilder::new();
    let mut routers = Vec::new();
    for (i, router) in config.routers().iter().enumerate() {
      let router = match router {
        HttpRouterConfig::RawRouter(r) => r,
        _ => unimplemented!(),
      };
      let router_component = resolve_ref(&app_config, router.operation().component())?;
      let router_binding = BoundComponent::new(i.to_string(), router_component);
      network = network.add_component(router_binding);
      routers.push(HttpRouter {
        path: router.path().to_owned(),
        operation: router.operation().operation().to_owned(),
        component: i.to_string(),
      });
    }
    debug!(?routers, "http routers");
    let network = network.build().await?;

    let instance = HttpInstance::new(network, routers, socket).await;

    Ok(instance)
  }
}

#[async_trait]
impl Trigger for Http {
  async fn run(
    &self,
    _name: String,
    app_config: AppConfiguration,
    config: TriggerDefinition,
    resources: Arc<HashMap<String, Resource>>,
  ) -> Result<(), RuntimeError> {
    debug!(kind = %TriggerKind::Http, "trigger:run");
    let config = if let TriggerDefinition::Http(config) = config {
      config
    } else {
      return Err(RuntimeError::InvalidTriggerConfig(TriggerKind::Http));
    };
    let resource_name = config.resource_id();
    let resource = resources
      .get(resource_name)
      .ok_or_else(|| RuntimeError::ResourceNotFound(TriggerKind::Http, resource_name.to_owned()))?;
    let socket = match resource {
      Resource::TcpPort(s) => s,
      _ => {
        return Err(RuntimeError::InvalidResourceType(
          TriggerKind::Http,
          ResourceKind::TcpPort,
          resource.kind(),
        ))
      }
    };

    let instance = self.handle_command(app_config, config, socket).await?;
    self.instance.lock().replace(instance);

    Ok(())
  }

  async fn shutdown_gracefully(self) -> Result<(), RuntimeError> {
    if self.instance.lock().is_none() {
      return Ok(());
    }
    let instance = self.instance.lock().take().unwrap();
    instance.shutdown().await?;
    Ok(())
  }

  async fn wait_for_done(&self) {
    info!("HTTP server waiting for SIGINT");
    tokio::signal::ctrl_c().await.unwrap();
    debug!("SIGINT received");
  }
}

impl fmt::Display for Http {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Cli Trigger",)
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;

  #[test_logger::test(tokio::test)]
  async fn test_decode() -> Result<()> {
    let request=b"\x88\xa6method\xa3Get\xa6scheme\xa4Http\xa9authority\xa0\xb0query_parameters\x80\xa4path\xa1/\xa3uri\xa1/\xa7version\xa6Http11\xa7headers\x82\xa6accept\x91\xa3*/*\xa4host\x91\xac0.0.0.0:8888";
    let req: wick_interface_http::HttpRequest = wasmrs_codec::messagepack::deserialize(request).unwrap();
    assert_eq!(req.path, "/");
    let response = b"\x83\xa7version\xa6Http11\xa6status\xa2Ok\xa7headers\x80";
    let res: wick_interface_http::HttpResponse = wasmrs_codec::messagepack::deserialize(response).unwrap();
    assert_eq!(res.status, wick_interface_http::StatusCode::Ok);
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_basic() -> Result<()> {
    let yaml = "
---
format: 1
resources:
    - name: http
      resource:
        kind: wick/resource/tcpport@v1
        port: 8888
        address: 0.0.0.0
triggers:
    - kind: wick/trigger/http@v1
      resource: http
      routers:
        - kind: wick/router/raw@v1
          path: /
          operation:
            name: http_handler
            component:
              kind: wick/component/wasmrs@v1
              ref: ../../integration/test-http-trigger-component/build/test_http_trigger_component.signed.wasm
    ";
    let app_config = AppConfiguration::from_yaml(yaml, &None)?;
    let trigger = Http::load_impl()?;
    let resource = Resource::new(app_config.resources().get("http").as_ref().unwrap().kind.clone())?;
    let resources = Arc::new([("http".to_owned(), resource)].iter().cloned().collect());
    let trigger_config = app_config.triggers()[0].clone();
    trigger
      .run("test".to_owned(), app_config, trigger_config, resources)
      .await?;
    let client = reqwest::Client::new();
    let res = client
      .post("http://0.0.0.0:8888")
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
}
