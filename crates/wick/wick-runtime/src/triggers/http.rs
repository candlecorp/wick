#![allow(clippy::needless_pass_by_value)]

mod component_utils;
mod conversions;
mod error;
mod middleware;
mod routers;
mod service_factory;

use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
pub(crate) use error::HttpError;
use futures::future::BoxFuture;
use hyper::{Body, Request, Response, Server};
use parking_lot::Mutex;
use routers::{HttpRouter, RawRouterHandler, RouterOperation};
use serde_json::json;
use structured_output::StructuredOutput;
use tokio::task::JoinHandle;
use tracing::Span;
use wick_config::config::{AppConfiguration, TriggerDefinition};

use super::{Trigger, TriggerKind};
use crate::dev::prelude::{RuntimeError, *};
use crate::resources::{Resource, ResourceKind};
use crate::triggers::http::service_factory::ServiceFactory;
use crate::Runtime;

trait RawRouter {
  fn handle(
    &self,
    tx_id: Uuid,
    remote_addr: SocketAddr,
    runtime: Runtime,
    request: Request<Body>,
    span: &Span,
  ) -> BoxFuture<Result<Response<Body>, HttpError>>;
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
  async fn new(runtime: Runtime, routers: Vec<HttpRouter>, initiating_span: &Span, socket: &SocketAddr) -> Self {
    let span = info_span!(parent:initiating_span,"http:server", %socket);

    span.in_scope(|| trace!(%socket,"http server starting"));
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let (running_tx, running_rx) = tokio::sync::oneshot::channel::<()>();
    let server = Server::bind(socket).serve(ServiceFactory::new(runtime, routers, span.id()));
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
        TriggerKind::Http.into(),
        "could not send shutdown signal; server may have already died".to_owned(),
      )
    })?;
    self.handle.await.map_err(|_| {
      RuntimeError::ShutdownFailed(
        TriggerKind::Http.into(),
        "waiting for server process to stop after sending shutdown signal failed".to_owned(),
      )
    })?;
    Ok(())
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
}

#[async_trait]
impl Trigger for Http {
  async fn run(
    &self,
    _name: String,
    runtime: Runtime,
    app_config: AppConfiguration,
    config: TriggerDefinition,
    resources: Arc<HashMap<String, Resource>>,
    span: Span,
  ) -> Result<StructuredOutput, RuntimeError> {
    span.in_scope(|| debug!(kind = %TriggerKind::Http, "trigger:run"));
    let config = if let config::TriggerDefinition::Http(config) = config {
      config
    } else {
      return Err(RuntimeError::TriggerKind(Context::Trigger, TriggerKind::Http));
    };
    let resource_name = config.resource();
    let resource = resources
      .get(resource_name)
      .ok_or_else(|| RuntimeError::ResourceNotFound(TriggerKind::Http.into(), resource_name.to_owned()))?;
    let socket = match resource {
      Resource::TcpPort(s) => *s,
      _ => {
        return Err(RuntimeError::InvalidResourceType(
          TriggerKind::Http.into(),
          ResourceKind::TcpPort,
          resource.kind(),
        ))
      }
    };

    let span = info_span!(parent: &span,"trigger:http:routers");

    let routers = span.in_scope(|| {
      let mut routers = Vec::new();
      for (i, router) in config.routers().iter().enumerate() {
        info!(path = router.path(), kind = %router.kind(), "registering http router");

        let router = match router {
          config::HttpRouterConfig::RawRouter(r) => routers::raw::register_raw_router(i, r)?,
          config::HttpRouterConfig::StaticRouter(r) => {
            routers::static_::register_static_router(i, resources.clone(), r)?
          }
          config::HttpRouterConfig::ProxyRouter(r) => routers::proxy::register_proxy_router(i, resources.clone(), r)?,
          config::HttpRouterConfig::RestRouter(r) => {
            routers::rest::register_rest_router(i, resources.clone(), &app_config, r)?
          }
        };

        routers.push(router);
      }
      debug!(?routers, "http routers");
      Ok::<_, RuntimeError>(routers)
    })?;

    let instance = HttpInstance::new(runtime, routers, &span, &socket).await;

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

#[cfg(test)]
mod test {

  // "port_limited" tests are grouped together and run on a single thread to prevent port contention
  mod port_limited {

    use anyhow::Result;

    use super::super::*;
    use crate::build_trigger_runtime;
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
      let rt = build_trigger_runtime(&app_config, Span::current())?.build(None).await?;

      let trigger = Http::default();
      let resource = Resource::new(app_config.resources().get(0).as_ref().unwrap().kind().clone())?;
      let resources = Arc::new([("http".to_owned(), resource)].iter().cloned().collect());
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
      let rt = build_trigger_runtime(&app_config, Span::current())?.build(None).await?;

      let trigger = Http::default();
      let resource = Resource::new(app_config.resources().get(0).as_ref().unwrap().kind().clone())?;
      let resources = Arc::new([("http".to_owned(), resource)].iter().cloned().collect());
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
      let rt = build_trigger_runtime(&app_config, Span::current())?.build(None).await?;

      let trigger = Http::default();
      let resource = Resource::new(app_config.resources().get(0).as_ref().unwrap().kind().clone())?;
      let resources = Arc::new([("http".to_owned(), resource)].iter().cloned().collect());
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

      let res: serde_json::Value = get("/api/this/FIRST_VALUE/some/222?third=third_a&fourth=true")
        .await?
        .json()
        .await?;

      println!("{:#?}", res);
      assert_eq!(
        res,
        json!({"first":"FIRST_VALUE", "second": 222,"third":["third_a"], "fourth":true })
      );

      let res: serde_json::Value = get("/api/this/FIRST_VALUE/some/222?third=third_a&third=third_b&fourth=true")
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
