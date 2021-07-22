use std::collections::HashMap;
use std::fmt::Display;

use log::error;
use serde::{
  Deserialize,
  Serialize,
};
use vino_codec::messagepack::{
  deserialize,
  rmp_deserialize,
};
use vino_component::{
  v0,
  Packet,
};

use crate::{
  Error,
  Result,
};

/// A HashMap mapping from a port name to a MessageTransport object.
pub type PortMap = HashMap<String, MessageTransport>;

/// The [MessageTransport] is the primary way messages are sent around Vino Networks, Schematics, and is the representation that normalizes output [Packet]'s.
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
  /// A message that contains a mapping of port names to encoded byte lists.
  MultiBytes(HashMap<String, Vec<u8>>),
  /// A map of port names to [MessageTransport] messages they are associated with.
  OutputMap(PortMap),
  /// A test message
  Test(String),
  /// An internal signal
  Signal(MessageSignal),
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

/// A simplified JSON representation of a MessageTransport
#[derive(Debug, Clone, Eq, Serialize, Deserialize, PartialEq)]
pub struct JsonOutput {
  pub error_msg: Option<String>,
  // #[serde(skip_serializing_if = "Option::is_none")]
  pub error_kind: JsonError,
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

impl MessageTransport {
  /// Returns `true` if the Message contains success data destined for a downstream
  /// consumer, false for Errors, Exceptions, and otherwise.
  #[must_use]
  pub fn is_ok(&self) -> bool {
    match self {
      MessageTransport::MessagePack(_) => true,
      MessageTransport::MultiBytes(_) => true,
      MessageTransport::Test(_) => true,
      MessageTransport::Exception(_) => false,
      MessageTransport::Error(_) => false,
      MessageTransport::Invalid => false,
      MessageTransport::OutputMap(_) => true,
      MessageTransport::Signal(_) => false,
    }
  }

  #[must_use]
  pub fn is_err(&self) -> bool {
    matches!(self, MessageTransport::Error(_))
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
      MessageTransport::MessagePack(bytes) => match deserialize::<serde_json::Value>(&bytes) {
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
      },
      _ => {
        error!(
          "Unhandled internal message trying to convert to JSON: {:?}",
          self
        );
        JsonOutput {
          value: serde_json::value::Value::Null,
          error_msg: Some("Unhandled internal message".to_owned()),
          error_kind: JsonError::InternalError,
        }
      }
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

  /// Attempts a conversion of a [MessageTransport] into the bytes of a [MessageTransport::MessagePack] variant
  pub fn into_bytes(self) -> Result<Vec<u8>> {
    match self {
      MessageTransport::MessagePack(v) => Ok(v),
      _ => Err(Error::PayloadConversionError("Invalid payload".to_owned())),
    }
  }

  /// Attempts a conversion of a [MessageTransport] into the [PortMap] of a [MessageTransport::OutputMap] variant
  pub fn into_output_map(self) -> Result<PortMap> {
    match self {
      MessageTransport::OutputMap(v) => Ok(v),
      _ => Err(Error::PayloadConversionError(
        "Invalid payload, not an output map".to_owned(),
      )),
    }
  }

  /// Attempts a conversion of a [MessageTransport] into a [HashMap<String, Vec<u8>>] from a [MessageTransport::MultiBytes] variant
  pub fn into_multibytes(self) -> Result<HashMap<String, Vec<u8>>> {
    match self {
      MessageTransport::MultiBytes(v) => Ok(v),
      _ => Err(Error::PayloadConversionError("Invalid payload".to_owned())),
    }
  }

  /// Try to deserialize a [MessageTransport] into the target type
  pub fn try_into<'de, T: Deserialize<'de>>(self) -> Result<T> {
    match self {
      MessageTransport::Invalid => Err(Error::Invalid),
      MessageTransport::Exception(v) => Err(Error::Exception(v)),
      MessageTransport::Error(v) => Err(Error::Error(v)),
      MessageTransport::MessagePack(buf) => {
        rmp_deserialize::<T>(&buf).map_err(Error::DeserializationError)
      }
      MessageTransport::MultiBytes(_) => todo!(),
      MessageTransport::OutputMap(_) => todo!(),
      MessageTransport::Test(_) => Err(Error::Invalid),
      MessageTransport::Signal(_) => Err(Error::Invalid),
    }
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
        v0::Payload::Close => MessageTransport::Signal(MessageSignal::Close),
        v0::Payload::OpenBracket => MessageTransport::Signal(MessageSignal::OpenBracket),
        v0::Payload::CloseBracket => MessageTransport::Signal(MessageSignal::CloseBracket),
      },
    }
  }
}
