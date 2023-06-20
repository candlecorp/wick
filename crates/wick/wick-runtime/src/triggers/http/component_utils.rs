use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;
use futures::TryStreamExt;
use hyper::header::CONTENT_LENGTH;
use hyper::http::response::Builder;
use hyper::http::{HeaderName, HeaderValue};
use hyper::{Body, Request, Response, StatusCode};
use serde_json::{Map, Value};
use tokio_stream::StreamExt;
use tracing::Span;
use wick_config::config::components::Codec;
use wick_interface_http::types as wick_http;
use wick_packet::{
  packets,
  Entity,
  InherentData,
  Invocation,
  Observer,
  Packet,
  PacketPayload,
  PacketStream,
  RuntimeConfig,
};

use super::conversions::convert_response;
use super::HttpError;
use crate::triggers::http::conversions::request_and_body_to_wick;
use crate::Runtime;

pub(super) async fn handle(
  target: Entity,
  codec: Codec,
  engine: Arc<Runtime>,
  req: Request<Body>,
  remote_addr: SocketAddr,
  config: Option<RuntimeConfig>,
) -> Result<PacketStream, HttpError> {
  let (tx, rx) = PacketStream::new_channels();

  let invocation = Invocation::new(
    Entity::server("http_client"),
    target,
    rx,
    InherentData::unsafe_default(),
    &Span::current(),
  );

  let stream = engine
    .invoke(invocation, config)
    .await
    .map_err(|e| HttpError::OperationError(e.to_string()));
  let (req, mut body) = request_and_body_to_wick(req, remote_addr)?;

  let packets = packets!(("request", req));
  for packet in packets {
    let _ = tx.send(packet);
  }
  tokio::spawn(async move {
    if codec == Codec::Json {
      let bytes: Result<Vec<Bytes>, _> = body.try_collect().await;
      match bytes {
        Ok(b) => {
          let bytes = b.join(&0);
          trace!(?bytes, "http:codec:json:bytes");
          let Ok(value) : Result<Value,_> = serde_json::from_slice(&bytes) else {
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

pub(super) enum Either {
  Request(wick_http::HttpRequest),
  Response(wick_http::HttpResponse),
}

pub(super) async fn handle_request_middleware(
  target: Entity,
  operation_config: Option<RuntimeConfig>,
  engine: Arc<Runtime>,
  req: &wick_http::HttpRequest,
) -> Result<Option<Either>, HttpError> {
  let packets = packets!(("request", req));
  let invocation = Invocation::new(
    Entity::server("http_client"),
    target,
    packets,
    InherentData::unsafe_default(),
    &Span::current(),
  );

  let mut stream = engine
    .invoke(invocation, operation_config)
    .await
    .map_err(|e| HttpError::OperationError(e.to_string()))?;

  let packet = stream.next().await;

  if packet.is_none() {
    return Ok(None);
  }

  let packet = packet.unwrap();

  if let Err(e) = packet {
    return Err(HttpError::InvalidPreRequestResponse(e.to_string()));
  }

  let packet = packet.unwrap();

  if packet.port() == "response" {
    let response: wick_http::HttpResponse = packet
      .decode()
      .map_err(|e| HttpError::InvalidPreRequestResponse(e.to_string()))?;

    Ok(Some(Either::Response(response)))
  } else if packet.port() == "request" {
    let request: wick_http::HttpRequest = packet
      .decode()
      .map_err(|e| HttpError::InvalidPreRequestResponse(e.to_string()))?;

    Ok(Some(Either::Request(request)))
  } else {
    Err(HttpError::InvalidPreRequestResponse("Invalid packet".to_owned()))
  }
}

pub(super) async fn handle_response_middleware(
  target: Entity,
  operation_config: Option<RuntimeConfig>,
  engine: Arc<Runtime>,
  req: &wick_http::HttpRequest,
  res: &wick_http::HttpResponse,
) -> Result<Option<wick_http::HttpResponse>, HttpError> {
  let packets = packets!(("request", req), ("response", res));
  let invocation = Invocation::new(
    Entity::server("http_client"),
    target,
    packets,
    InherentData::unsafe_default(),
    &Span::current(),
  );

  let mut stream = engine
    .invoke(invocation, operation_config)
    .await
    .map_err(|e| HttpError::OperationError(e.to_string()))?;

  let packet = stream.next().await;

  if packet.is_none() {
    return Ok(None);
  }

  let packet = packet.unwrap();

  if let Err(e) = packet {
    return Err(HttpError::InvalidPostRequestResponse(e.to_string()));
  }

  let packet = packet.unwrap();

  if packet.port() == "response" {
    let response: wick_http::HttpResponse = packet
      .decode()
      .map_err(|e| HttpError::InvalidPostRequestResponse(e.to_string()))?;

    Ok(Some(response))
  } else {
    Err(HttpError::InvalidPostRequestResponse("Invalid packet".to_owned()))
  }
}

pub(super) async fn respond(
  codec: Codec,
  stream: Result<PacketStream, HttpError>,
) -> Result<Response<Body>, HttpError> {
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
          if let PacketPayload::Err(e) = p.payload() {
            return Err(HttpError::OutputStream(p.port().to_owned(), e.msg().to_owned()));
          }
          if p.is_done() {
            continue;
          }
          let response: wick_interface_http::types::HttpResponse = p
            .decode()
            .map_err(|e| HttpError::Deserialize("response".to_owned(), e.to_string()))?;
          builder = convert_response(builder, response)?;
        } else if p.port() == "body" {
          if let PacketPayload::Err(e) = p.payload() {
            return Err(HttpError::OutputStream(p.port().to_owned(), e.msg().to_owned()));
          }
          if !p.has_data() {
            continue;
          }
          if codec == Codec::Json {
            let response: Value = p.decode().map_err(|e| HttpError::Codec(codec, e.to_string()))?;
            let as_str = response.to_string();
            let bytes = as_str.as_bytes();
            body.extend_from_slice(bytes);
          } else {
            let response: Bytes = p.decode().map_err(|e| HttpError::Codec(codec, e.to_string()))?;
            body.extend_from_slice(&response);
          }
        }
      }
      Err(e) => return Err(HttpError::OperationError(e.to_string())),
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

enum MapVal {
  RawVal(Value),
  RootArray(Vec<Value>),
}

pub(super) async fn stream_to_json(stream: PacketStream) -> Result<Value, HttpError> {
  let mut stream = stream;
  let mut map = HashMap::new();
  while let Some(packet) = stream.next().await {
    match packet {
      Ok(p) => {
        if let PacketPayload::Err(err) = p.payload() {
          return Err(HttpError::OutputStream(p.port().to_owned(), err.msg().to_owned()));
        }
        if !p.has_data() {
          continue;
        }
        let port = p.port().to_owned();
        if let Some(val) = map.remove(p.port()) {
          let val = match val {
            MapVal::RawVal(v) => {
              let response: Value = p
                .decode_value()
                .map_err(|e| HttpError::Codec(Codec::Json, e.to_string()))?;
              MapVal::RootArray(vec![v, response])
            }
            MapVal::RootArray(mut v) => {
              let response: Value = p
                .decode_value()
                .map_err(|e| HttpError::Codec(Codec::Json, e.to_string()))?;
              v.push(response);
              MapVal::RootArray(v)
            }
          };
          map.insert(port, val);
        } else {
          let response: Value = p
            .decode_value()
            .map_err(|e| HttpError::Codec(Codec::Json, e.to_string()))?;
          map.insert(port, MapVal::RawVal(response));
        }
      }
      Err(e) => return Err(HttpError::OperationError(e.to_string())),
    }
  }
  let json = Value::Object(
    map
      .into_iter()
      .map(|(k, v)| {
        (
          k,
          match v {
            MapVal::RawVal(v) => v,
            MapVal::RootArray(v) => Value::Array(v),
          },
        )
      })
      .collect::<Map<String, Value>>(),
  );
  Ok(json)
}
