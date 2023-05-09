#![allow(unreachable_code, unused)]
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures::future::BoxFuture;
use futures::{Future, TryStreamExt};
use hyper::header::CONTENT_LENGTH;
use hyper::http::response::Builder;
use hyper::http::uri::InvalidUri;
use hyper::http::{HeaderName, HeaderValue};
use hyper::server::conn::AddrStream;
use hyper::service::Service;
use hyper::{Body, HeaderMap, Request, Response, StatusCode};
use tokio_stream::StreamExt;
use wick_config::config::components::Codec;
use wick_interface_http::wasmrs::PayloadError;
use wick_packet::{packets, Entity, Invocation, Observer, Packet, PacketStream};

use super::{ComponentRouterHandler, HttpError, HttpRouter};
use crate::dev::prelude::RuntimeError;
use crate::triggers::http::conversions::{convert_response, request_to_wick};
use crate::Runtime;

pub(super) struct ServiceFactory {
  engine: Arc<Runtime>,
  routers: Arc<Vec<HttpRouter>>,
}

impl ServiceFactory {
  pub(super) fn new(engine: Runtime, routers: Vec<HttpRouter>) -> Self {
    Self {
      engine: Arc::new(engine),
      routers: Arc::new(routers),
    }
  }
}

impl Service<&AddrStream> for ServiceFactory {
  type Response = ResponseService;
  type Error = hyper::Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + Sync>>;

  fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, conn: &AddrStream) -> Self::Future {
    let engine = self.engine.clone();
    let routers = self.routers.clone();

    let remote_addr = conn.remote_addr();

    let fut = async move { Ok(ResponseService::new(remote_addr, engine, routers)) };
    Box::pin(fut)
  }
}

pub(super) struct ResponseService {
  remote_addr: SocketAddr,
  engine: Arc<Runtime>,
  routers: Arc<Vec<HttpRouter>>,
}

impl ResponseService {
  fn new(remote_addr: SocketAddr, engine: Arc<Runtime>, routers: Arc<Vec<HttpRouter>>) -> Self {
    Self {
      remote_addr,
      engine,
      routers,
    }
  }
}

async fn respond(codec: Codec, stream: Result<PacketStream, RuntimeError>) -> Result<Response<Body>, HttpError> {
  if let Err(e) = stream {
    return Ok(
      Builder::new()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(e.to_string()))
        .unwrap(),
    );
  }
  let mut stream = stream.unwrap();
  let mut builder = Response::builder();
  let mut body = bytes::BytesMut::new();
  while let Some(packet) = stream.next().await {
    match packet {
      Ok(p) => {
        if p.port() == "response" {
          if p.is_done() {
            continue;
          }
          let response: wick_interface_http::types::HttpResponse = p
            .deserialize()
            .map_err(|e| HttpError::Deserialize("response".to_owned(), e.to_string()))?;
          builder = convert_response(builder, response)?;
        } else if p.port() == "body" {
          if !p.has_data() {
            continue;
          }
          if codec == Codec::Json {
            let response: serde_json::Value = p.deserialize().map_err(|e| HttpError::InvalidResponse(e.to_string()))?;
            let as_str = response.to_string();
            let bytes = as_str.as_bytes();
            body.extend_from_slice(bytes);
          } else {
            let response: Bytes = p.deserialize().map_err(|e| HttpError::InvalidResponse(e.to_string()))?;
            body.extend_from_slice(&response);
          }
        }
      }
      Err(e) => return Err(HttpError::InvalidResponse(e.to_string())),
    }
  }
  builder = reset_header(builder, CONTENT_LENGTH, body.len());
  Ok(builder.body(body.freeze().into()).unwrap())
}

fn reset_header(mut builder: Builder, header: HeaderName, value: impl Into<HeaderValue>) -> Builder {
  #[allow(clippy::option_if_let_else)]
  if let Some(headers) = builder.headers_mut() {
    if let Some(cl) = headers.get_mut(&header) {
      *cl = value.into();
    } else {
      headers.insert(header, value.into());
    }
  } else {
    builder = builder.header(header, value.into());
  };
  builder
}

impl Service<Request<Body>> for ResponseService {
  type Response = Response<Body>;
  type Error = hyper::Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

  fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, req: Request<Body>) -> Self::Future {
    let engine = self.engine.clone();
    let router = self
      .routers
      .iter()
      .find(|r| req.uri().path().starts_with(r.path()))
      .cloned();
    let remote_addr = self.remote_addr;

    Box::pin(async move {
      match router {
        Some(h) => match h {
          HttpRouter::Component(h) => {
            let codec = h.codec;
            let handler = handle_component_router(h, engine, req);
            match handler {
              Ok(handler) => {
                match respond(
                  codec,
                  handler.await.map_err(|e| RuntimeError::TriggerFailed(e.to_string())),
                )
                .await
                {
                  Ok(r) => Ok(r),
                  Err(e) => Ok(make_ise(e)),
                }
              }
              Err(e) => Ok(make_ise(e)),
            }
          }
          HttpRouter::Raw(r) => match r.component.handle(remote_addr, req).await {
            Ok(res) => Ok(res),
            Err(e) => Ok(make_ise(e)),
          },
        },
        None => Ok(make_ise("")),
      }
    })
  }
}

fn make_ise(e: impl std::fmt::Display) -> Response<Body> {
  Builder::new()
    .status(StatusCode::INTERNAL_SERVER_ERROR)
    .body(Body::from(e.to_string()))
    .unwrap()
}

fn handle_component_router(
  h: ComponentRouterHandler,
  engine: Arc<Runtime>,
  req: Request<Body>,
) -> Result<BoxFuture<'static, Result<PacketStream, HttpError>>, RuntimeError> {
  let task = Box::pin(async move {
    let (tx, rx) = PacketStream::new_channels();
    let invocation = Invocation::new(
      Entity::server("http_client"),
      Entity::operation(&h.component, &h.operation),
      None,
    );
    let stream = engine
      .invoke(invocation, rx, None)
      .await
      .map_err(|e| HttpError::OperationError(e.to_string()));
    match request_to_wick(req) {
      Ok((req, mut body)) => {
        let packets = packets!(("request", req));
        for packet in packets {
          let _ = tx.send(packet);
        }
        tokio::spawn(async move {
          if h.codec == Codec::Json {
            let bytes: Result<Vec<Bytes>, _> = body.try_collect().await;
            match bytes {
              Ok(b) => {
                let bytes = b.join(&0);
                trace!(?bytes, "http:codec:json:bytes");
                let Ok(value) : Result<serde_json::Value,_> = serde_json::from_slice(&bytes) else {
                  let _ = tx.error(wick_packet::Error::component_error("Could not decode body as JSON"));
                  return;
                };
                let _ = tx.send(Packet::encode("body", value));
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
                  let _ = tx.send(Packet::encode("body", b));
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
      Err(e) => Err(e),
    }
  });
  Ok(task)
}
