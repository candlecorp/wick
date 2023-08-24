use std::net::SocketAddr;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::future::BoxFuture;
use futures::TryStreamExt;
use hyper::service::Service;
use hyper::{Body, Request, Response};
use serde_json::Value;
use tokio_stream::StreamExt;
use tracing::{Instrument, Span};
use uuid::Uuid;
use wick_config::config::{Codec, RawRouterConfig, WickRouter};
use wick_packet::{packets, Base64Bytes, Entity, InherentData, Invocation, Observer, Packet, PacketStream};

use crate::dev::prelude::RuntimeError;
use crate::triggers::http::component_utils::respond;
use crate::triggers::http::conversions::request_and_body_to_wick;
use crate::triggers::http::middleware::resolve_middleware_components;
use crate::triggers::http::{HttpError, HttpRouter, RawRouter, RawRouterHandler, RouterOperation};
use crate::triggers::ComponentId;
use crate::Runtime;

#[derive()]
#[must_use]
pub(super) struct RawComponentRouter {
  config: Arc<RouterOperation>,
}

impl RawComponentRouter {
  pub(super) fn new(config: RouterOperation) -> Self {
    Self {
      config: Arc::new(config),
    }
  }
}

impl RawRouter for RawComponentRouter {
  fn handle(
    &self,
    tx_id: Uuid,
    remote_addr: SocketAddr,
    runtime: Runtime,
    request: Request<Body>,
    span: &Span,
  ) -> BoxFuture<Result<Response<Body>, HttpError>> {
    let handler = RawHandler::new(tx_id, self.config.clone(), runtime, remote_addr);
    let span = info_span!(parent: span, "raw");

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
  tx_id: Uuid,
  config: Arc<RouterOperation>,
  runtime: Runtime,
  remote_addr: SocketAddr,
}

impl RawHandler {
  fn new(tx_id: Uuid, config: Arc<RouterOperation>, runtime: Runtime, remote_addr: SocketAddr) -> Self {
    RawHandler {
      tx_id,
      config,
      runtime,
      remote_addr,
    }
  }

  /// Serve a request.
  async fn serve(self, req: Request<Body>) -> Result<Response<Body>, HttpError> {
    let entity = Entity::operation(&self.config.component, &self.config.operation);
    let codec = self.config.codec;
    let stream = self.handle(entity, codec, req).await;
    respond(codec, stream).await
  }

  async fn handle(self, target: Entity, codec: Codec, req: Request<Body>) -> Result<PacketStream, HttpError> {
    let Self {
      tx_id,
      remote_addr,
      config,
      runtime,
    } = self;

    let (tx, rx) = PacketStream::new_channels();

    let invocation = Invocation::new_with_id(
      tx_id,
      Entity::server("http_client"),
      target,
      rx,
      InherentData::unsafe_default(),
      &Span::current(),
    );

    let stream = runtime
      .invoke(invocation, config.config.clone())
      .await
      .map_err(|e| HttpError::OperationError(e.to_string()));
    let (mut req, mut body) = request_and_body_to_wick(req, remote_addr)?;
    req.path = req.path.trim_start_matches(&config.path).to_owned();

    let packets = packets!(("request", req));
    for packet in packets {
      let _ = tx.send(packet);
    }
    tokio::spawn(async move {
      if codec == Codec::Json {
        let bytes: Result<Vec<bytes::Bytes>, _> = body.try_collect().await;
        match bytes {
          Ok(b) => {
            let bytes = b.join(&0);
            trace!(?bytes, "http:codec:json:bytes");
            serde_json::from_slice::<Value>(&bytes).map_or_else(
              |_| {
                let _ = tx.send(Packet::err("body", "Could not decode body as JSON"));
              },
              |value| {
                let _ = tx.send(Packet::encode("body", value));
              },
            );
          }
          Err(e) => {
            let _ = tx.send(Packet::err("body", e.to_string()));
          }
        }
      } else {
        while let Some(bytes) = body.next().await {
          trace!(?bytes, "http:codec:raw:bytes");
          match bytes {
            Ok(b) => {
              let _ = tx.send(Packet::encode("body", Base64Bytes::new(b)));
            }
            Err(e) => {
              let _ = tx.send(Packet::err("body", e.to_string()));
            }
          }
        }
      }
      trace!("http:request:done");
      let _ = tx.send(Packet::done("body"));
    });
    stream
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

pub(crate) fn register_raw_router(index: usize, router_config: &RawRouterConfig) -> Result<HttpRouter, RuntimeError> {
  trace!(index, "registering raw router");
  let middleware = resolve_middleware_components(router_config)?;

  let component_id = router_config.operation().component_id()?;

  let router = RouterOperation {
    operation: router_config.operation().name().to_owned(),
    component: component_id.to_owned(),
    codec: router_config.codec().copied().unwrap_or_default(),
    config: router_config.operation().config().and_then(|v| v.value().cloned()),
    path: router_config.path().to_owned(),
  };

  let router = RawComponentRouter::new(router);

  Ok(HttpRouter::Raw(RawRouterHandler {
    path: router_config.path().to_owned(),
    component: Arc::new(router),
    middleware,
  }))
}
