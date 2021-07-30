use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::pin::Pin;
use std::task::Poll;

use log::error;
use serde::de::DeserializeOwned;
use serde::{
  Deserialize,
  Serialize,
};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_stream::Stream;
use vino_codec::{
  json,
  messagepack,
  raw,
};
use vino_component::{
  v0,
  InvocationPacket,
  Packet,
};

use crate::{
  Error,
  Result,
};

/// A HashMap mapping from a port name to a MessageTransport object.
pub type PortMap = HashMap<String, MessageTransport>;

/// The [MessageTransport] is the primary way messages are sent around Vino Networks, Schematics, and is the representation that normalizes output [Packet]'s.
#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageTransport {
  /// An invalid message.
  Invalid,
  /// A message carrying an exception.
  Exception(String),
  /// A message carrying an error.
  Error(String),
  /// A message carrying a MessagePack encoded list of bytes.
  MessagePack(Vec<u8>),
  /// A test message
  Test(String),
  /// An internal signal
  Signal(MessageSignal),
  /// A success value in an intermediary format
  Success(serde_value::Value),
  /// A JSON String
  Json(String),
}

/// Signals that need to be handled before propagating to a downstream consumer.
#[derive(Debug, Clone, Copy, Eq, Serialize, Deserialize, PartialEq)]
pub enum MessageSignal {
  /// Indicates this channel is closing and should not be polled any further.
  Close,
  /// Indicates that a message is coming down in chunks and this is the start.
  OpenBracket,
  /// Indicates a chunked message has been completed.
  CloseBracket,
}

impl Default for MessageTransport {
  fn default() -> Self {
    Self::Invalid
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[must_use]
/// A wrapper for a map of String to [MessageTransport]
pub struct TransportMap(HashMap<String, MessageTransport>);

impl TransportMap {
  /// Constructor for [TransportMap] with initial map
  pub fn with_map(map: HashMap<String, MessageTransport>) -> Self {
    Self(map)
  }

  /// Constructor for an empty [TransportMap]
  pub fn new() -> Self {
    Self(HashMap::new())
  }

  /// Deserialize a JSON Object into a [TransportMap]
  pub fn from_json_str(json: &str) -> Result<Self> {
    let json: HashMap<String, serde_value::Value> = json::deserialize(json).map_err(de_err)?;
    Ok(TransportMap::with_map(
      json
        .into_iter()
        .map(|(name, val)| (name, MessageTransport::Success(val)))
        .collect(),
    ))
  }

  /// Insert a MessageTransport by port name
  pub fn insert<T: AsRef<str>>(
    &mut self,
    port: T,
    msg: MessageTransport,
  ) -> Option<MessageTransport> {
    self.0.insert(port.as_ref().to_owned(), msg)
  }

  /// Get a reference to the [MessageTransport] behind the passed port
  #[must_use]
  pub fn get(&self, port: &str) -> Option<&MessageTransport> {
    self.0.get(port)
  }

  /// Remove a key from the map and attempt to deserialize it into the destination type
  pub fn consume<T: DeserializeOwned>(&mut self, key: &str) -> Result<T> {
    let v = self.0.remove(key).ok_or_else(|| {
      Error::DeserializationError(format!("TransportMap does not have field '{}'", key))
    })?;
    let e = Err(Error::DeserializationError(format!(
      "Payload could not be converted to destination type. Payload was: {:?}",
      v
    )));
    match v {
      MessageTransport::Invalid => e,
      MessageTransport::Exception(_) => e,
      MessageTransport::Error(_) => e,
      MessageTransport::Test(_) => e,
      MessageTransport::Signal(_) => e,
      MessageTransport::MessagePack(bytes) => messagepack::deserialize(&bytes).map_err(de_err),
      MessageTransport::Success(v) => raw::deserialize(v).map_err(de_err),
      MessageTransport::Json(v) => json::deserialize(&v).map_err(de_err),
    }
  }

  /// Returns true if any of the held messages is an error or exception type.
  #[must_use]
  pub fn has_error(&self) -> bool {
    for msg in self.0.values() {
      if msg.is_err() {
        return true;
      }
    }
    false
  }

  /// Returns an error if the transport is holding one, otherwise returns None.
  #[must_use]
  pub fn take_error(self) -> Option<MessageTransport> {
    for (_, v) in self.0 {
      if v.is_err() {
        return Some(v);
      }
    }
    None
  }

  /// Returns the inner [HashMap]
  #[must_use]
  pub fn into_inner(self) -> HashMap<String, MessageTransport> {
    self.0
  }

  /// Attempts to normalize the [TransportMap] into messagepacked bytes by serializing success formats or throwing an error.
  pub fn try_into_messagepack_bytes(self) -> Result<HashMap<String, Vec<u8>>> {
    let mut map = HashMap::new();
    let err = Error::SerializationError(
      "Transport map contained payloads that could not be converted to bytes.".to_owned(),
    );
    for (k, v) in self.0 {
      let e = Err(err.clone());
      let bytes = match v {
        MessageTransport::Invalid => e,
        MessageTransport::Exception(_) => e,
        MessageTransport::Error(_) => e,
        MessageTransport::Test(_) => e,
        MessageTransport::Signal(_) => e,
        MessageTransport::MessagePack(bytes) => Ok(bytes),
        MessageTransport::Success(v) => {
          let bytes = messagepack::serialize(&v).map_err(ser_err)?;
          Ok(bytes)
        }
        MessageTransport::Json(v) => {
          let value: serde_value::Value = json::deserialize(&v).map_err(de_err)?;
          let bytes = messagepack::serialize(&value).map_err(ser_err)?;
          Ok(bytes)
        }
      }?;
      map.insert(k, bytes);
    }
    Ok(map)
  }
}

fn ser_err<T: Display>(e: T) -> Error {
  Error::SerializationError(e.to_string())
}

fn de_err<T: Display>(e: T) -> Error {
  Error::DeserializationError(e.to_string())
}

/// A simplified JSON representation of a MessageTransport
#[derive(Debug, Clone, Eq, Serialize, Deserialize, PartialEq)]
pub struct JsonOutput {
  /// Error message for the port if it exists.
  pub error_msg: Option<String>,
  #[serde(skip_serializing_if = "JsonError::is_none")]
  /// The error kind if it exists.
  pub error_kind: JsonError,
  /// The return value.
  pub value: serde_json::Value,
}

/// The kinds of errors that a [JsonOutput] can carry
#[derive(Debug, Clone, Copy, Eq, Serialize, Deserialize, PartialEq)]
pub enum JsonError {
  /// No error
  None,
  /// A message from a [MessageTransport::Exception]
  Exception,
  /// A message from a [MessageTransport::Error]
  Error,
  /// An error originating internally
  InternalError,
}

impl Display for JsonError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      JsonError::None => "None",
      JsonError::Exception => "Exception",
      JsonError::Error => "Error",
      JsonError::InternalError => "InternalError",
    };
    f.write_str(s)
  }
}

impl JsonError {
  #[must_use]
  /// This is analagous to Option::is_none for a [JsonError] kind
  pub fn is_none(&self) -> bool {
    matches!(self, JsonError::None)
  }
  #[must_use]
  /// This is analagous to Option::is_some for a [JsonError] kind
  pub fn is_some(&self) -> bool {
    !matches!(self, JsonError::None)
  }
}

fn unhandled_conversion(transport: &MessageTransport) -> JsonOutput {
  error!("Unhandled  JSON conversion: {:?}", transport);
  JsonOutput {
    value: serde_json::value::Value::Null,
    error_msg: Some(format!("Internal error converting {:?} to JSON", transport)),
    error_kind: JsonError::InternalError,
  }
}

fn handle_result_conversion(result: std::result::Result<serde_json::Value, String>) -> JsonOutput {
  match result {
    Ok(payload) => JsonOutput {
      value: payload,
      error_msg: None,
      error_kind: JsonError::None,
    },
    Err(e) => {
      let msg = format!(
        "Error deserializing messagepack payload to JSON value: {:?}",
        e
      );
      error!("{}", msg);
      JsonOutput {
        value: serde_json::value::Value::Null,
        error_msg: Some(msg),
        error_kind: JsonError::InternalError,
      }
    }
  }
}

impl MessageTransport {
  /// Returns `true` if the Message contains success data destined for a downstream
  /// consumer, false for Errors, Exceptions, and otherwise.
  #[must_use]
  pub fn is_ok(&self) -> bool {
    match self {
      MessageTransport::MessagePack(_) => true,
      MessageTransport::Json(_) => true,
      MessageTransport::Test(_) => true,
      MessageTransport::Success(_) => true,
      MessageTransport::Exception(_) => false,
      MessageTransport::Error(_) => false,
      MessageTransport::Invalid => false,
      MessageTransport::Signal(_) => false,
    }
  }

  #[must_use]
  /// Returns true if the [MessageTransport] is holding an Error or Exception variant.
  pub fn is_err(&self) -> bool {
    matches!(
      self,
      MessageTransport::Error(_) | MessageTransport::Exception(_)
    )
  }

  /// Converts a [MessageTransport] into [serde_json::Value] representation of a [JsonOutput]
  #[must_use]
  pub fn into_json(self) -> serde_json::Value {
    let output = match self {
      MessageTransport::Invalid => JsonOutput {
        value: serde_json::value::Value::Null,
        error_msg: Some("Invalid value".to_owned()),
        error_kind: JsonError::Error,
      },
      MessageTransport::Exception(v) => JsonOutput {
        value: serde_json::value::Value::Null,
        error_msg: Some(v),
        error_kind: JsonError::Exception,
      },
      MessageTransport::Error(v) => JsonOutput {
        value: serde_json::value::Value::Null,
        error_msg: Some(v),
        error_kind: JsonError::Error,
      },
      MessageTransport::MessagePack(bytes) => handle_result_conversion(
        messagepack::deserialize::<serde_json::Value>(&bytes).map_err(|e| e.to_string()),
      ),
      MessageTransport::Success(v) => handle_result_conversion(
        raw::deserialize::<serde_json::Value>(v).map_err(|e| e.to_string()),
      ),
      MessageTransport::Json(v) => handle_result_conversion(
        json::deserialize::<serde_json::Value>(&v).map_err(|e| e.to_string()),
      ),
      MessageTransport::Test(_) => unhandled_conversion(&self),
      MessageTransport::Signal(_) => unhandled_conversion(&self),
    };

    let mut map = serde_json::Map::new();
    map.insert("value".to_owned(), output.value);
    if let Some(msg) = output.error_msg {
      map.insert("error_msg".to_owned(), serde_json::Value::String(msg));
    }
    map.insert(
      "error_kind".to_owned(),
      serde_json::Value::String(output.error_kind.to_string()),
    );
    serde_json::value::Value::Object(map)
  }

  /// Creates a [MessageTransport] by serializing a passed object with messagepack
  pub fn messagepack<T: ?Sized + Serialize>(item: &T) -> Self {
    match messagepack::serialize(item) {
      Ok(bytes) => Self::MessagePack(bytes),
      Err(e) => Self::Error(format!(
        "Error serializing into messagepack: {}",
        e.to_string()
      )),
    }
  }

  /// Creates a [MessageTransport] by serializing a passed object into a raw intermediary format
  pub fn success<T: Serialize>(item: &T) -> Self {
    match raw::serialize(item) {
      Ok(v) => Self::Success(v),
      Err(e) => Self::Error(format!(
        "Error serializing into raw intermediary format: {}",
        e.to_string()
      )),
    }
  }

  /// Creates a [MessageTransport] by serializing a passed object into JSON
  pub fn json<T: Serialize>(item: &T) -> Self {
    match json::serialize(item) {
      Ok(v) => Self::Json(v),
      Err(e) => Self::Error(format!("Error serializing into json: {}", e.to_string())),
    }
  }

  /// A utility function for [MessageTransport::Signal(MessageSignal::Close)]
  pub fn close() -> Self {
    MessageTransport::Signal(MessageSignal::Close)
  }

  /// Attempts a conversion of a [MessageTransport] into the bytes of a [MessageTransport::MessagePack] variant
  pub fn into_bytes(self) -> Result<Vec<u8>> {
    match self {
      MessageTransport::MessagePack(v) => Ok(v),
      _ => Err(Error::PayloadConversionError("Invalid payload".to_owned())),
    }
  }

  /// Try to deserialize a [MessageTransport] into the target type
  pub fn try_into<T: DeserializeOwned>(self) -> Result<T> {
    match self {
      MessageTransport::Invalid => Err(Error::Invalid),
      MessageTransport::Exception(v) => Err(Error::Exception(v)),
      MessageTransport::Error(v) => Err(Error::Error(v)),
      MessageTransport::MessagePack(buf) => messagepack::rmp_deserialize::<T>(&buf)
        .map_err(|e| Error::DeserializationError(e.to_string())),
      MessageTransport::Success(v) => {
        raw::raw_deserialize::<T>(v).map_err(|e| Error::DeserializationError(e.to_string()))
      }
      MessageTransport::Json(v) => {
        json::json_deserialize::<T>(&v).map_err(|e| Error::DeserializationError(e.to_string()))
      }
      MessageTransport::Test(_) => Err(Error::Invalid),
      MessageTransport::Signal(_) => Err(Error::Invalid),
    }
  }

  /// Convert a [HashMap<String, MessageTransport>] into a [serde_json::value::Map]
  #[must_use]
  pub fn map_to_json(
    raw_result: HashMap<String, MessageTransport>,
  ) -> serde_json::value::Map<String, serde_json::Value> {
    raw_result
      .into_iter()
      .map(|(k, payload)| {
        (
          k,
          payload.try_into().unwrap_or_else(|e: Error| {
            serde_json::json!({
              "error": format!("Internal error: {:?}, invalid format", e.to_string())
            })
          }),
        )
      })
      .collect()
  }
}

impl From<Vec<u8>> for MessageTransport {
  fn from(v: Vec<u8>) -> Self {
    MessageTransport::MessagePack(v)
  }
}

impl From<&Vec<u8>> for MessageTransport {
  fn from(v: &Vec<u8>) -> Self {
    MessageTransport::MessagePack(v.clone())
  }
}

impl From<&[u8]> for MessageTransport {
  fn from(v: &[u8]) -> Self {
    MessageTransport::MessagePack(v.to_vec())
  }
}

impl From<Packet> for MessageTransport {
  fn from(output: Packet) -> MessageTransport {
    match output {
      Packet::V0(v) => match v {
        v0::Payload::Exception(v) => MessageTransport::Exception(v),
        v0::Payload::Error(v) => MessageTransport::Error(v),
        v0::Payload::Invalid => MessageTransport::Invalid,
        v0::Payload::MessagePack(bytes) => MessageTransport::MessagePack(bytes),
        v0::Payload::Json(v) => MessageTransport::Json(v),
        v0::Payload::Success(v) => MessageTransport::Success(v),
        v0::Payload::Close => MessageTransport::Signal(MessageSignal::Close),
        v0::Payload::OpenBracket => MessageTransport::Signal(MessageSignal::OpenBracket),
        v0::Payload::CloseBracket => MessageTransport::Signal(MessageSignal::CloseBracket),
      },
    }
  }
}

/// A [MessageTransportStream] is a stream of [MessageTransport]s
#[derive(Debug)]
pub struct MessageTransportStream {
  // port_statuses: HashMap<String, PortStatus>,
  rx: RefCell<UnboundedReceiver<InvocationTransport>>,
}

impl MessageTransportStream {
  #[doc(hidden)]
  #[must_use]
  pub fn new(rx: UnboundedReceiver<InvocationTransport>) -> Self {
    Self {
      rx: RefCell::new(rx),
    }
  }
}

lazy_static::lazy_static! {
    static ref CLOSE_MESSAGE: MessageTransport = {
      MessageTransport::Signal(MessageSignal::Close)
    };
}

impl Stream for MessageTransportStream {
  type Item = InvocationTransport;

  fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
    let mut rx = self.rx.borrow_mut();
    match rx.poll_recv(cx) {
      Poll::Ready(Some(wrapper)) => {
        if wrapper.port == crate::SYSTEM_ID && CLOSE_MESSAGE.eq(&wrapper.payload) {
          Poll::Ready(None)
        } else {
          Poll::Ready(Some(wrapper))
        }
      }
      other => other,
    }
  }
}

/// An [InvocationTransport] is a wrapper around a [MessageTransport] with the port name and invocation id embedded in it.
#[derive(Debug, Clone, PartialEq)]
pub struct InvocationTransport {
  /// The port the message originated from
  pub port: String,
  /// The port's output
  pub payload: MessageTransport,
}

impl From<InvocationPacket> for InvocationTransport {
  fn from(p: InvocationPacket) -> Self {
    Self {
      port: p.port,
      payload: p.payload.into(),
    }
  }
}
