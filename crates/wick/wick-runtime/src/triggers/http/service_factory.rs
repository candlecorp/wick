#![allow(unreachable_code, unused)]
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::Future;
use hyper::http::response::Builder;
use hyper::service::Service;
use hyper::{Body, Request, Response, StatusCode};
use tokio_stream::StreamExt;
use wick_packet::{packets, Entity, Invocation, Observer, Packet, PacketStream};

use super::{HttpError, HttpRouter};
use crate::dev::prelude::RuntimeError;
use crate::triggers::http::conversions::{convert_request, convert_response};
use crate::Engine;

pub(super) struct ServiceFactory {
  engine: Arc<Engine>,
  routers: Arc<Vec<HttpRouter>>,
}

impl ServiceFactory {
  pub(super) fn new(engine: Engine, routers: Vec<HttpRouter>) -> Self {
    Self {
      engine: Arc::new(engine),
      routers: Arc::new(routers),
    }
  }
}

impl<T> Service<T> for ServiceFactory {
  type Response = ResponseService;
  type Error = hyper::Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + Sync>>;

  fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, _: T) -> Self::Future {
    let engine = self.engine.clone();
    let routers = self.routers.clone();

    let fut = async move { Ok(ResponseService::new(engine, routers)) };
    Box::pin(fut)
  }
}

pub(super) struct ResponseService {
  engine: Arc<Engine>,
  routers: Arc<Vec<HttpRouter>>,
}

impl ResponseService {
  fn new(engine: Arc<Engine>, routers: Arc<Vec<HttpRouter>>) -> Self {
    Self { engine, routers }
  }
}

async fn respond(stream: Result<PacketStream, RuntimeError>) -> Result<Response<Body>, HttpError> {
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
          let response: wick_interface_http::HttpResponse = p
            .deserialize()
            .map_err(|e| HttpError::Deserialize("response".to_owned(), e.to_string()))?;
          builder = convert_response(builder, response)?;
        } else if p.port() == "body" {
          if p.is_done() {
            continue;
          }
          let response: bytes::Bytes = p
            .deserialize()
            .map_err(|e| HttpError::Deserialize("body".to_owned(), e.to_string()))?;
          body.extend_from_slice(&response);
        }
      }
      Err(e) => return Err(HttpError::InvalidResponse(e.to_string())),
    }
  }
  Ok(builder.body(body.freeze().into()).unwrap())
}

impl Service<Request<Body>> for ResponseService {
  type Response = Response<Body>;
  type Error = hyper::Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

  fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, req: Request<Body>) -> Self::Future {
    trace!("http:trigger:request");
    let engine = self.engine.clone();
    let router = self
      .routers
      .iter()
      .find(|r| req.uri().path().starts_with(&r.path))
      .cloned();
    trace!(?router, "http:trigger:request");

    Box::pin(async move {
      let (tx, rx) = PacketStream::new_channels();
      let handler = match router {
        Some(h) => {
          let invocation = Invocation::new(
            Entity::server("http_client"),
            Entity::operation(&h.component, &h.operation),
            None,
          );
          let stream = engine.invoke(invocation, rx);
          match convert_request(req) {
            Ok((req, mut body)) => {
              let packets = packets!(("request", req));
              for packet in packets {
                let _ = tx.send(packet);
              }
              tokio::spawn(async move {
                while let Some(bytes) = body.next().await {
                  match bytes {
                    Ok(b) => {
                      let _ = tx.send(Packet::encode("body", b));
                    }
                    Err(e) => {
                      let _ = tx.send(Packet::err("body", e.to_string()));
                    }
                  }
                }
                let _ = tx.send(Packet::done("body"));
              });
              Ok(stream)
            }
            Err(e) => Err(e),
          }
        }
        None => Err(HttpError::NotFound(req.uri().path().to_owned())),
      };

      match handler {
        Ok(handler) => match respond(handler.await).await {
          Ok(r) => Ok(r),
          Err(e) => Ok(
            Builder::new()
              .status(StatusCode::INTERNAL_SERVER_ERROR)
              .body(Body::from(e.to_string()))
              .unwrap(),
          ),
        },
        Err(e) => Ok(
          Builder::new()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(e.to_string()))
            .unwrap(),
        ),
      }
    })
  }
}
