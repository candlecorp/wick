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
use wick_config::config::Codec;
use wick_interface_http::types as wick_http;
use wick_packet::{
  packets,
  Base64Bytes,
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
  path_prefix: &str,
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
  let (mut req, mut body) = request_and_body_to_wick(req, remote_addr)?;
  req.path = req.path.trim_start_matches(path_prefix).to_owned();

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

pub(super) async fn handle_request_middleware(
  target: Entity,
  operation_config: Option<RuntimeConfig>,
  engine: Arc<Runtime>,
  req: &wick_http::HttpRequest,
) -> Result<Option<wick_http::RequestMiddlewareResponse>, HttpError> {
  let packets = packets!(("request", req));
  let invocation = Invocation::new(
    Entity::server("http_client"),
    target.clone(),
    packets,
    InherentData::unsafe_default(),
    &Span::current(),
  );

  let stream = engine
    .invoke(invocation, operation_config)
    .await
    .map_err(|e| HttpError::OperationError(e.to_string()))?;

  let packets = stream.collect::<Result<Vec<Packet>, _>>().await;

  let packets = match packets {
    Ok(packets) => packets,
    Err(e) => return Err(HttpError::InvalidPreRequestResponse(e.to_string())),
  };

  if let Some(packet) = packets.iter().find(|p| p.is_error()) {
    return Err(HttpError::InvalidPreRequestResponse(
      packet.clone().unwrap_err().msg().to_owned(),
    ));
  };

  let Some(packet) = packets.into_iter().find(|p| p.has_data()) else {
    return Err(HttpError::PreRequestResponseNoData(target));
  };

  if packet.port() == "output" {
    let response: wick_http::RequestMiddlewareResponse = packet
      .decode()
      .map_err(|e| HttpError::InvalidPreRequestResponse(e.to_string()))?;

    Ok(Some(response))
  } else {
    Err(HttpError::InvalidPreRequestResponse(format!(
      "Invalid response named {}, pre-request middleware expects a response named 'output' that is either an HttpRequest or an HttpResponse",
      packet.port()
    )))
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
    target.clone(),
    packets,
    InherentData::unsafe_default(),
    &Span::current(),
  );

  let stream = engine
    .invoke(invocation, operation_config)
    .await
    .map_err(|e| HttpError::OperationError(e.to_string()))?;

  let packets = stream.collect::<Result<Vec<Packet>, _>>().await;

  let packets = match packets {
    Ok(packets) => packets,
    Err(e) => return Err(HttpError::InvalidPostRequestResponse(e.to_string())),
  };

  if let Some(packet) = packets.iter().find(|p| p.is_error()) {
    return Err(HttpError::InvalidPostRequestResponse(
      packet.clone().unwrap_err().msg().to_owned(),
    ));
  };

  let Some(packet) = packets.into_iter().find(|p| p.has_data()) else {
    return Err(HttpError::PostRequestResponseNoData(target));
  };

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
            let response: Base64Bytes = p.decode().map_err(|e| HttpError::Bytes(e.to_string()))?;
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
