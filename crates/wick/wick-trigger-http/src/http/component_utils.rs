use std::collections::HashMap;

use futures::stream::StreamExt;
use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::http::response::Builder;
use hyper::http::{HeaderName, HeaderValue};
use hyper::{Body, Response, StatusCode};
use serde_json::{Map, Value};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio::sync::oneshot;
use tracing::Span;
use uuid::Uuid;
use wick_config::config::Codec;
use wick_interface_http::types::{self as wick_http};
use wick_packet::{
  packets,
  Base64Bytes,
  Entity,
  InherentData,
  Invocation,
  Packet,
  PacketExt,
  PacketPayload,
  PacketStream,
  RuntimeConfig,
};
use wick_runtime::Runtime;

use super::conversions::convert_response;
use super::HttpError;

pub(super) async fn handle_request_middleware(
  tx_id: Uuid,
  target: Entity,
  operation_config: Option<RuntimeConfig>,
  runtime: Runtime,
  req: &wick_http::HttpRequest,
  span: &Span,
) -> Result<Option<wick_http::RequestMiddlewareResponse>, HttpError> {
  let packets = packets!(("request", req));
  let invocation = Invocation::new_with_id(
    tx_id,
    Entity::server("http_client"),
    target.clone(),
    packets,
    InherentData::unsafe_default(),
    span,
  );

  let stream = runtime
    .invoke(invocation, operation_config)
    .await
    .map_err(|e| HttpError::OperationError(e.to_string()))?;

  let packets = stream
    .collect::<Vec<Result<Packet, _>>>()
    .await
    .into_iter()
    .collect::<Result<Vec<_>, _>>();

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
  tx_id: Uuid,
  target: Entity,
  operation_config: Option<RuntimeConfig>,
  runtime: Runtime,
  req: &wick_http::HttpRequest,
  res: &wick_http::HttpResponse,
  span: &Span,
) -> Result<Option<wick_http::HttpResponse>, HttpError> {
  let packets = packets!(("request", req), ("response", res));
  let invocation = Invocation::new_with_id(
    tx_id,
    Entity::server("http_client"),
    target.clone(),
    packets,
    InherentData::unsafe_default(),
    span,
  );

  let stream = runtime
    .invoke(invocation, operation_config)
    .await
    .map_err(|e| HttpError::OperationError(e.to_string()))?;

  let packets = stream
    .collect::<Vec<Result<Packet, _>>>()
    .await
    .into_iter()
    .collect::<Result<Vec<_>, _>>();

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
  let stream = stream.unwrap();
  let builder = Response::builder();

  let (handle, response, mut body_stream) = split_stream(stream);

  let response = match response.await {
    Ok(response) => response?,
    Err(e) => {
      handle.abort();
      return Ok(
        Builder::new()
          .status(StatusCode::INTERNAL_SERVER_ERROR)
          .body(Body::from(e.to_string()))
          .unwrap(),
      );
    }
  };

  let mut builder = convert_response(builder, response)?;
  let event_stream = builder
    .headers_ref()
    .and_then(|h| h.get(CONTENT_TYPE))
    .map_or(false, |v| v == "text/event-stream");

  let res = if event_stream {
    let (tx, rx) = unbounded_channel();
    let _output_handle = tokio::spawn(async move {
      while let Some(p) = body_stream.recv().await {
        match codec {
          Codec::Json => {
            let chunk = p
              .decode::<wick_http::HttpEvent>()
              .map_err(|e| HttpError::Bytes(e.to_string()))
              .map(|v| to_sse_string_bytes(&v));
            let _ = tx.send(chunk);
          }
          Codec::Raw => {
            let chunk = p
              .decode::<Base64Bytes>()
              .map_err(|e| HttpError::Bytes(e.to_string()))
              .map(Into::into);
            let _ = tx.send(chunk);
          }
          Codec::Text => {
            let chunk = p
              .decode::<String>()
              .map_err(|e| HttpError::Utf8Text(e.to_string()))
              .map(Into::into);
            let _ = tx.send(chunk);
          }
          Codec::FormData => unreachable!("FormData is not supported as a decoder for HTTP responses"),
        }
      }
    });
    let body = Body::wrap_stream(tokio_stream::wrappers::UnboundedReceiverStream::new(rx));
    builder.body(body).unwrap()
  } else {
    let mut body = bytes::BytesMut::new();
    while let Some(p) = body_stream.recv().await {
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
    builder = reset_header(builder, CONTENT_LENGTH, body.len());
    builder.body(body.freeze().into()).unwrap()
  };

  Ok(res)
}

fn split_stream(
  mut stream: PacketStream,
) -> (
  tokio::task::JoinHandle<()>,
  oneshot::Receiver<Result<wick_http::HttpResponse, HttpError>>,
  UnboundedReceiver<Packet>,
) {
  let (body_tx, body_rx) = unbounded_channel();
  let (res_tx, res_rx) = oneshot::channel();
  let mut res_tx = Some(res_tx);

  let handle = tokio::spawn(async move {
    while let Some(packet) = stream.next().await {
      match packet {
        Ok(p) => {
          if p.port() == "response" {
            if p.is_done() {
              continue;
            }
            let Some(sender) = res_tx.take() else {
              // we only respect the first packet to the response port.
              continue;
            };
            if let PacketPayload::Err(e) = p.payload() {
              let _ = sender.send(Err(HttpError::OutputStream(p.port().to_owned(), e.msg().to_owned())));
              break;
            }
            let response: Result<wick_http::HttpResponse, _> = p
              .decode()
              .map_err(|e| HttpError::Deserialize("response".to_owned(), e.to_string()));
            let _ = sender.send(response);
          } else if p.port() == "body" {
            let _ = body_tx.send(p);
          }
        }
        Err(e) => {
          if let Some(sender) = res_tx.take() {
            let _ = sender.send(Err(HttpError::OperationError(e.to_string())));
          }
          warn!(?e, "http:stream:error");
          break;
        }
      }
    }
  });

  (handle, res_rx, body_rx)
}

fn to_sse_string_bytes(event: &wick_http::HttpEvent) -> Vec<u8> {
  let mut sse_string = String::new();

  if !event.event.is_empty() {
    sse_string.push_str("event: ");
    sse_string.push_str(&event.event);
    sse_string.push('\n');
  }

  // Splitting data by newline to ensure each line is prefixed with "data: "
  for line in event.data.split('\n') {
    sse_string.push_str("data: ");
    sse_string.push_str(line);
    sse_string.push('\n');
  }

  if !event.id.is_empty() {
    sse_string.push_str("id: ");
    sse_string.push_str(&event.id);
    sse_string.push('\n');
  }

  if let Some(ref retry) = event.retry {
    sse_string.push_str("retry: ");
    sse_string.push_str(&retry.to_string());
    sse_string.push('\n');
  }

  // Adding the required empty line to separate events
  sse_string.push('\n');

  sse_string.into()
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
