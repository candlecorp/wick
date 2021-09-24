/// Module for the [crate::MessageTransport], [crate::TransportWrapper], and the JSON
/// representations of each.
pub mod stream;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Display;

use log::error;
use serde::de::DeserializeOwned;
use serde::{
  Deserialize,
  Serialize,
};
use vino_codec::{
  json,
  messagepack,
  raw,
};
use vino_packet::{
  v0,
  v1,
  Packet,
  PacketWrapper,
};

use crate::error::TransportError;
use crate::{
  Error,
  Result,
};

lazy_static::lazy_static! {
  /// A static close message
  pub static ref CLOSE_MESSAGE: MessageTransport = {
    MessageTransport::Signal(MessageSignal::Done)
  };

  /// A static system close message
  pub static ref SYSTEM_CLOSE_MESSAGE: TransportWrapper = {
    TransportWrapper::new(
      crate::SYSTEM_ID,
      MessageTransport::Signal(MessageSignal::Done)
    )
  };
  /// An error representing there was a major problem in
  /// the MessageTransport deserialization.
  pub static ref JSON_ERROR: serde_json::Value = {
    let error = TransportJson {
      value: serde_json::value::Value::Null,
      signal: None,
      error_msg: Some("Error serializing packet into JSON.".to_owned()),
      error_kind: JsonError::InternalError,
    };
    match json::to_value(&error) {
      Ok(v) => v,
      Err(e) => {
        panic!("Error creating an internal error! Error was : {}", e)
      }
    }
  };
}

/// The [MessageTransport] is the primary way messages are sent around Vino Networks, Schematics, and is the representation that normalizes output [Packet]'s.
#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageTransport {
  /// TODO
  #[serde(rename = "0")]
  Success(Success),

  /// TODO
  #[serde(rename = "1")]
  Failure(Failure),

  #[serde(rename = "3")]
  /// An internal signal
  Signal(MessageSignal),
}

/// TODO
#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Success {
  #[serde(rename = "0")]
  /// A message carrying a MessagePack encoded list of bytes.
  MessagePack(Vec<u8>),

  #[serde(rename = "1")]
  /// A success value in an intermediary format
  Serialized(serde_value::Value),

  #[serde(rename = "2")]
  /// A JSON String
  Json(String),
}

/// TODO
#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Failure {
  #[serde(rename = "0")]
  /// An invalid message.
  Invalid,

  #[serde(rename = "1")]
  /// A message carrying an exception.
  Exception(String),

  #[serde(rename = "2")]
  /// A message carrying an error.
  Error(String),
}

/// Signals that need to be handled before propagating to a downstream consumer.
#[derive(Debug, Clone, Copy, Eq, Serialize, Deserialize, PartialEq)]
pub enum MessageSignal {
  /// Indicates the job that opened this port is finished with it.
  Done,

  /// Indicates that a message is coming down in chunks and this is the start.
  OpenBracket,

  /// Indicates a chunked message has been completed.
  CloseBracket,
}

impl MessageTransport {
  /// Returns `true` if the Message contains success data destined for a downstream
  /// consumer, `false` for Errors, Exceptions, and otherwise.
  #[must_use]
  pub fn is_ok(&self) -> bool {
    matches!(self, MessageTransport::Success(_))
  }

  #[must_use]
  /// Returns true if the [MessageTransport] is holding an Error or Exception variant.
  pub fn is_err(&self) -> bool {
    matches!(self, MessageTransport::Failure(_))
  }

  #[must_use]
  /// Returns true if the [MessageTransport] is a [MessageTransport::Signal] variant.
  pub fn is_signal(&self) -> bool {
    matches!(self, Self::Signal(_))
  }

  /// Converts a [MessageTransport] into [serde_json::Value]
  /// representation of a [TransportJson]
  #[must_use]
  pub fn into_json(self) -> serde_json::Value {
    let output = match self {
      MessageTransport::Success(success) => match success {
        Success::MessagePack(bytes) => handle_result_conversion(
          messagepack::deserialize::<serde_json::Value>(&bytes).map_err(|e| e.to_string()),
        ),
        Success::Serialized(v) => handle_result_conversion(
          raw::deserialize::<serde_json::Value>(v).map_err(|e| e.to_string()),
        ),
        Success::Json(v) => handle_result_conversion(
          json::deserialize::<serde_json::Value>(&v).map_err(|e| e.to_string()),
        ),
      },
      MessageTransport::Failure(failure) => match failure {
        Failure::Invalid => TransportJson {
          value: serde_json::value::Value::Null,
          signal: None,
          error_msg: Some("Invalid value".to_owned()),
          error_kind: JsonError::Error,
        },
        Failure::Exception(v) => TransportJson {
          value: serde_json::value::Value::Null,
          signal: None,
          error_msg: Some(v),
          error_kind: JsonError::Exception,
        },
        Failure::Error(v) => TransportJson {
          value: serde_json::value::Value::Null,
          signal: None,
          error_msg: Some(v),
          error_kind: JsonError::Error,
        },
      },
      MessageTransport::Signal(s) => TransportJson {
        value: serde_json::value::Value::Null,
        signal: Some(s),
        error_msg: None,
        error_kind: JsonError::None,
      },
    };

    json::to_value(&output).unwrap_or_else(|_| JSON_ERROR.clone())
  }

  /// Creates a [MessageTransport] by serializing a passed object with messagepack
  pub fn messagepack<T: ?Sized + Serialize>(item: &T) -> Self {
    match messagepack::serialize(item) {
      Ok(bytes) => Self::Success(Success::MessagePack(bytes)),
      Err(e) => Self::Failure(Failure::Error(format!(
        "Error serializing into messagepack: {}",
        e.to_string()
      ))),
    }
  }

  /// Creates a [MessageTransport] by serializing a passed object into a raw intermediary format
  pub fn success<T: Serialize>(item: &T) -> Self {
    match raw::serialize(item) {
      Ok(v) => Self::Success(Success::Serialized(v)),
      Err(e) => Self::Failure(Failure::Error(format!(
        "Error serializing into raw intermediary format: {}",
        e.to_string()
      ))),
    }
  }

  /// Creates a [MessageTransport] by serializing a passed object into JSON
  pub fn json<T: Serialize>(item: &T) -> Self {
    match json::serialize(item) {
      Ok(v) => Self::Success(Success::Json(v)),
      Err(e) => Self::Failure(Failure::Error(format!(
        "Error serializing into json: {}",
        e.to_string()
      ))),
    }
  }

  /// Creates a [MessageTransport::Failure(Failure::Error)] with the passed message.
  pub fn error<T: AsRef<str>>(msg: T) -> Self {
    Self::Failure(Failure::Error(msg.as_ref().to_owned()))
  }

  /// Creates a [MessageTransport::Failure(Failure::Exception)] with the passed message.
  pub fn exception<T: AsRef<str>>(msg: T) -> Self {
    Self::Failure(Failure::Exception(msg.as_ref().to_owned()))
  }

  /// A utility function for [MessageTransport::Signal(MessageSignal::Done)]
  pub fn done() -> Self {
    MessageTransport::Signal(MessageSignal::Done)
  }

  /// Try to deserialize a [MessageTransport] into the target type
  pub fn try_into<T: DeserializeOwned>(self) -> Result<T> {
    match self {
      Self::Success(success) => match success {
        Success::MessagePack(v) => messagepack::rmp_deserialize::<T>(&v)
          .map_err(|e| Error::DeserializationError(e.to_string())),
        Success::Serialized(v) => {
          raw::raw_deserialize::<T>(v).map_err(|e| Error::DeserializationError(e.to_string()))
        }
        Success::Json(v) => {
          json::json_deserialize::<T>(&v).map_err(|e| Error::DeserializationError(e.to_string()))
        }
      },
      Self::Failure(failure) => match failure {
        Failure::Invalid => Err(Error::Invalid),
        Failure::Exception(v) => Err(Error::Exception(v)),
        Failure::Error(v) => Err(Error::Error(v)),
      },
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

impl From<Packet> for MessageTransport {
  fn from(output: Packet) -> MessageTransport {
    match output {
      Packet::V0(v) => match v {
        v0::Payload::Exception(v) => MessageTransport::Failure(Failure::Exception(v)),
        v0::Payload::Error(v) => MessageTransport::Failure(Failure::Error(v)),
        v0::Payload::Invalid => MessageTransport::Failure(Failure::Invalid),
        v0::Payload::MessagePack(bytes) => MessageTransport::Success(Success::MessagePack(bytes)),
        v0::Payload::Json(v) => MessageTransport::Success(Success::Json(v)),
        v0::Payload::Success(v) => MessageTransport::Success(Success::Serialized(v)),
        v0::Payload::Done => MessageTransport::Signal(MessageSignal::Done),
        v0::Payload::OpenBracket => MessageTransport::Signal(MessageSignal::OpenBracket),
        v0::Payload::CloseBracket => MessageTransport::Signal(MessageSignal::CloseBracket),
      },
      Packet::V1(v) => match v {
        vino_packet::v1::Payload::Success(success) => match success {
          vino_packet::v1::Success::MessagePack(bytes) => {
            MessageTransport::Success(Success::MessagePack(bytes))
          }
          vino_packet::v1::Success::Success(v) => MessageTransport::Success(Success::Serialized(v)),
          vino_packet::v1::Success::Json(v) => MessageTransport::Success(Success::Json(v)),
        },
        vino_packet::v1::Payload::Failure(failure) => match failure {
          vino_packet::v1::Failure::Invalid => MessageTransport::Failure(Failure::Invalid),
          vino_packet::v1::Failure::Exception(v) => {
            MessageTransport::Failure(Failure::Exception(v))
          }
          vino_packet::v1::Failure::Error(v) => MessageTransport::Failure(Failure::Error(v)),
        },
        vino_packet::v1::Payload::Signal(signal) => match signal {
          vino_packet::v1::Signal::Done => MessageTransport::Signal(MessageSignal::Done),
          vino_packet::v1::Signal::OpenBracket => todo!(),
          vino_packet::v1::Signal::CloseBracket => todo!(),
        },
      },
    }
  }
}

impl From<MessageTransport> for Packet {
  fn from(output: MessageTransport) -> Packet {
    match output {
      MessageTransport::Success(success) => match success {
        Success::MessagePack(v) => Packet::V1(v1::Payload::Success(v1::Success::MessagePack(v))),
        Success::Serialized(v) => Packet::V1(v1::Payload::Success(v1::Success::Success(v))),
        Success::Json(v) => Packet::V1(v1::Payload::Success(v1::Success::Json(v))),
      },
      MessageTransport::Failure(failure) => match failure {
        Failure::Invalid => Packet::V1(v1::Payload::Failure(v1::Failure::Invalid)),
        Failure::Exception(m) => Packet::V1(v1::Payload::Failure(v1::Failure::Exception(m))),
        Failure::Error(m) => Packet::V1(v1::Payload::Failure(v1::Failure::Error(m))),
      },
      MessageTransport::Signal(signal) => match signal {
        MessageSignal::Done => Packet::V1(v1::Payload::Signal(v1::Signal::Done)),
        MessageSignal::OpenBracket => Packet::V1(v1::Payload::Signal(v1::Signal::OpenBracket)),
        MessageSignal::CloseBracket => Packet::V1(v1::Payload::Signal(v1::Signal::CloseBracket)),
      },
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[must_use]
/// A wrapper for a map of [String]s to [MessageTransport]
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
    if json.trim() == "" {
      Ok(TransportMap::new())
    } else {
      let json: HashMap<String, TransportJson> = json::deserialize(json).map_err(de_err)?;
      Ok(TransportMap::with_map(
        json
          .into_iter()
          .map(|(name, val)| (name, val.into()))
          .collect(),
      ))
    }
  }

  /// Turn a list of "field=value" strings into a [TransportMap] of [MessageTransport::Json] items.
  pub fn from_kv_json(values: &[String]) -> Result<Self> {
    let mut payload = TransportMap::new();
    for input in values {
      match input.split_once("=") {
        Some((name, value)) => {
          debug!("PORT:'{}', VALUE:'{}'", name, value);
          payload.insert(
            name,
            MessageTransport::Success(Success::Json(value.to_owned())),
          );
        }
        None => {
          return Err(Error::DeserializationError(format!(
            "Invalid port=value pair: '{}'",
            input
          )))
        }
      }
    }
    Ok(payload)
  }

  /// Insert a [MessageTransport] by port name
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

  /// Remove a key from the held map and attempt to deserialize it into the destination type
  pub fn consume<T: DeserializeOwned>(&mut self, key: &str) -> Result<T> {
    let v = self.0.remove(key).ok_or_else(|| {
      Error::DeserializationError(format!("TransportMap does not have field '{}'", key))
    })?;
    let e = Err(Error::DeserializationError(format!(
      "Payload could not be converted to destination type. Payload was: {:?}",
      v
    )));
    match v {
      MessageTransport::Success(success) => match success {
        Success::MessagePack(bytes) => messagepack::deserialize(&bytes).map_err(de_err),
        Success::Serialized(v) => raw::deserialize(v).map_err(de_err),
        Success::Json(v) => json::deserialize(&v).map_err(de_err),
      },
      MessageTransport::Failure(_) => e,
      MessageTransport::Signal(_) => e,
    }
  }

  /// Remove a key from the held map and return the raw [MessageTransport].
  pub fn consume_raw(&mut self, key: &str) -> Result<MessageTransport> {
    self.0.remove(key).ok_or_else(|| {
      Error::DeserializationError(format!("TransportMap does not have field '{}'", key))
    })
  }

  /// Transpose any ports named "output" to "input". This is for a better user experience when
  /// trying to pipe components together without a full runtime. This should never be done
  /// without also providing a way to turn it off.
  #[doc(hidden)]
  pub fn transpose_output_name(&mut self) {
    let output = self.0.remove("output");
    if let Some(msg) = output {
      debug!("Transposing [output] to [input]");
      self.0.insert("input".to_owned(), msg);
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

  /// Attempts to normalize the [TransportMap] into messagepacked bytes
  /// by serializing success formats or throwing an error.
  pub fn try_into_messagepack_bytes(self) -> Result<HashMap<String, Vec<u8>>> {
    let mut map = HashMap::new();
    for (k, v) in self.0 {
      let bytes = match v {
        MessageTransport::Success(success) => match success {
          Success::MessagePack(bytes) => Ok(bytes),
          Success::Serialized(v) => {
            let bytes = messagepack::serialize(&v).map_err(ser_err)?;
            Ok(bytes)
          }
          Success::Json(v) => {
            let value: serde_value::Value = json::deserialize(&v).map_err(de_err)?;
            let bytes = messagepack::serialize(&value).map_err(ser_err)?;
            Ok(bytes)
          }
        },
        MessageTransport::Failure(failure) => match failure {
          Failure::Invalid => Err(Error::SerializationError(
            "Refusing to serialize an invalid payload".to_owned(),
          )),
          Failure::Exception(e) => Err(Error::SerializationError(format!(
            "Exceptions need to be processed by a runtime, not sent to components. Error was: {}",
            e
          ))),
          Failure::Error(e) => Err(Error::SerializationError(format!(
            "Errors need to be processed by a runtime, not sent to components. Error was: {}",
            e
          ))),
        },
        MessageTransport::Signal(_) => Err(Error::SerializationError(
          "Signal messages need to be processed by a runtime, not sent to components.".to_owned(),
        )),
      }?;
      map.insert(k, bytes);
    }
    Ok(map)
  }

  /// Merge another [TransportMap] into the calling map.
  pub fn merge(&mut self, map: TransportMap) {
    for (k, v) in map.into_inner() {
      self.insert(k, v);
    }
  }
}

impl<K, V> TryFrom<&HashMap<K, V>> for TransportMap
where
  K: AsRef<str> + Send + Sync,
  V: Serialize + Sync,
{
  type Error = TransportError;

  fn try_from(v: &HashMap<K, V>) -> Result<Self> {
    let serialized_data: HashMap<String, MessageTransport> = v
      .iter()
      .map(|(k, v)| {
        Ok((
          k.as_ref().to_owned(),
          MessageTransport::Success(Success::MessagePack(
            messagepack::serialize(&v).map_err(ser_err)?,
          )),
        ))
      })
      .filter_map(Result::ok)
      .collect();

    let payload = TransportMap::with_map(serialized_data);
    Ok(payload)
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
#[must_use]
pub struct TransportJson {
  /// Error message for the port if it exists.
  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error_msg: Option<String>,

  /// The error kind if it exists.
  #[serde(default)]
  #[serde(skip_serializing_if = "JsonError::is_none")]
  pub error_kind: JsonError,

  /// The Signal if the message was a [MessageTransport::Signal]
  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub signal: Option<MessageSignal>,

  /// The return value.
  pub value: serde_json::Value,
}

impl From<TransportJson> for MessageTransport {
  fn from(v: TransportJson) -> Self {
    match v.error_kind {
      JsonError::None => match v.signal {
        Some(signal) => MessageTransport::Signal(signal),
        None => {
          // We just parsed JSON and are now turning part of it
          // back into JSON which doesn't feel good. This is only
          // used for command line testing and piping but if it ends
          // up being used for more it will need to be better handled.
          MessageTransport::Success(Success::Json(v.value.to_string()))
        }
      },
      JsonError::Exception => match v.error_msg {
        Some(err) => MessageTransport::Failure(Failure::Exception(err)),
        None => MessageTransport::Failure(Failure::Exception(
          "<No message passed with exception>".to_owned(),
        )),
      },
      JsonError::Error => match v.error_msg {
        Some(err) => MessageTransport::Failure(Failure::Error(err)),
        None => MessageTransport::Failure(Failure::Error(
          "<No message passed with exception>".to_owned(),
        )),
      },
      JsonError::InternalError => {
        MessageTransport::Failure(Failure::Error("Internal Error (10001)".to_owned()))
      }
    }
  }
}

/// The kinds of errors that a [TransportJson] can carry
#[derive(Debug, Clone, Copy, Eq, Serialize, Deserialize, PartialEq)]
#[must_use]
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

impl Default for JsonError {
  fn default() -> Self {
    JsonError::None
  }
}

impl Display for JsonError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      JsonError::None => "None",
      JsonError::Exception => "Exception",
      JsonError::Error => "Error",
      JsonError::InternalError => "Internal Error",
    };
    f.write_str(s)
  }
}

impl JsonError {
  #[must_use]
  /// This is analogous to Option::is_none for a [JsonError] kind
  pub fn is_none(&self) -> bool {
    matches!(self, JsonError::None)
  }
  #[must_use]
  /// This is analogous to Option::is_some for a [JsonError] kind
  pub fn is_some(&self) -> bool {
    !matches!(self, JsonError::None)
  }
}

fn handle_result_conversion(
  result: std::result::Result<serde_json::Value, String>,
) -> TransportJson {
  match result {
    Ok(payload) => TransportJson {
      value: payload,
      signal: None,
      error_msg: None,
      error_kind: JsonError::None,
    },
    Err(e) => {
      let msg = format!(
        "Error deserializing messagepack payload to JSON value: {:?}",
        e
      );
      error!("{}", msg);
      TransportJson {
        value: serde_json::value::Value::Null,
        signal: None,
        error_msg: Some(msg),
        error_kind: JsonError::InternalError,
      }
    }
  }
}

impl Display for MessageTransport {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if let MessageTransport::Signal(signal) = self {
      return write!(f, "Signal({})", signal.to_string());
    }
    write!(
      f,
      "{}",
      match self {
        MessageTransport::Failure(v) => v.to_string(),
        MessageTransport::Signal(v) => v.to_string(),
        MessageTransport::Success(v) => v.to_string(),
      }
    )
  }
}

impl Display for MessageSignal {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "{}",
      match self {
        MessageSignal::Done => "Done",
        MessageSignal::OpenBracket => "OpenBracket",
        MessageSignal::CloseBracket => "CloseBracket",
      }
    ))
  }
}
impl Display for Failure {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "{}",
      match self {
        Failure::Invalid => "Invalid",
        Failure::Exception(_) => "Exception",
        Failure::Error(_) => "Error",
      }
    ))
  }
}
impl Display for Success {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "{}",
      match self {
        Success::MessagePack(_) => "MessagePack",
        Success::Serialized(_) => "Success",
        Success::Json(_) => "JSON",
      }
    ))
  }
}

/// A [TransportWrapper] is a wrapper around a [MessageTransport] with the port name it originated from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[must_use]
pub struct TransportWrapper {
  /// The port the message originated from.
  pub port: String,
  /// The port's output.
  pub payload: MessageTransport,
}

impl TransportWrapper {
  /// Constructor for [TransportWrapper]s.
  pub fn new(port: &str, payload: MessageTransport) -> Self {
    Self {
      port: port.to_owned(),
      payload,
    }
  }

  /// Constructor for a [TransportWrapper] with a port of [crate::COMPONENT_ERROR] indicating an internal error occurred.
  pub fn internal_error(payload: MessageTransport) -> Self {
    Self {
      port: crate::COMPONENT_ERROR.to_owned(),
      payload,
    }
  }

  /// Attempt to deserialize the contained [MessageTransport] into the destination value.
  pub fn try_into<T: DeserializeOwned>(self) -> Result<T> {
    self.payload.try_into()
  }

  /// Converts the embedded [MessageTransport] into a [serde_json::Value::Object]
  /// map of port names to [TransportJson]s
  #[must_use]
  pub fn into_json(self) -> serde_json::Value {
    let payload = self.payload.into_json();

    let mut map = serde_json::Map::new();
    map.insert(self.port, payload);

    serde_json::value::Value::Object(map)
  }
}

impl From<PacketWrapper> for TransportWrapper {
  fn from(p: PacketWrapper) -> Self {
    Self {
      port: p.port,
      payload: p.payload.into(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test_env_log::test]
  fn serializes_done() -> Result<()> {
    let close = MessageTransport::done();
    let value = close.into_json();
    println!("Value: {}", value);
    assert_eq!(value.to_string(), r#"{"signal":"Done","value":null}"#);
    Ok(())
  }

  #[test_env_log::test]
  fn test_merge() -> Result<()> {
    let mut map1 = TransportMap::new();
    map1.insert("first", MessageTransport::success(&"first-val"));
    let mut map2 = TransportMap::new();
    map2.insert("second", MessageTransport::success(&"second-val"));
    map1.merge(map2);
    let val1: String = map1.consume("first")?;
    assert_eq!(val1, "first-val");
    let val2: String = map1.consume("second")?;
    assert_eq!(val2, "second-val");

    Ok(())
  }
}
