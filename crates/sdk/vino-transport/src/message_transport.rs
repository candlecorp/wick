/// Module for the [crate::MessageTransport], [crate::TransportWrapper], and the JSON
/// representations of each.
#[cfg(feature = "async")]
pub(super) mod stream;

/// JSON-related module.
#[cfg(feature = "json")]
pub(super) mod transport_json;

/// The module for the TransportMap, a Port->[MessageTransport] map that serves as input to a component invocation.
pub(super) mod transport_map;

/// The module for TransportWrapper, a struct that includes the port a [MessageTransport] originated from.
pub(super) mod transport_wrapper;

use std::fmt::Display;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
#[cfg(feature = "json")]
use vino_codec::json;
use vino_codec::messagepack;
#[cfg(feature = "raw")]
use vino_codec::raw;
use vino_packet::{v0, v1, Packet};

use crate::{Error, Result};

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
  #[cfg(feature = "raw")]
  /// A success value in an intermediary format
  Serialized(serde_value::Value),

  #[serde(rename = "2")]
  #[cfg(feature = "json")]
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

  /// Converts the [MessageTransport] into a messagepack-compatible transport.
  pub fn to_messagepack(&mut self) {
    match &self {
      Self::Success(Success::MessagePack(_)) => {}
      #[cfg(feature = "raw")]
      Self::Success(Success::Serialized(v)) => *self = Self::messagepack(&v),
      #[cfg(feature = "json")]
      Self::Success(Success::Json(json)) => {
        *self = match json::deserialize::<serde_value::Value>(json) {
          Ok(val) => Self::messagepack(&val),
          Err(e) => Self::error(format!(
            "Could not convert JSON payload to MessagePack: {}",
            e
          )),
        }
      }
      _ => {}
    };
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
    #[cfg(feature = "raw")]
    match raw::serialize(item) {
      Ok(v) => Self::Success(Success::Serialized(v)),
      Err(e) => Self::Failure(Failure::Error(format!(
        "Error serializing into raw intermediary format: {}",
        e.to_string()
      ))),
    }
    #[cfg(not(feature = "raw"))]
    match messagepack::serialize(item) {
      Ok(v) => Self::Success(Success::MessagePack(v)),
      Err(e) => Self::Failure(Failure::Error(format!(
        "Error serializing into messagepack format: {}",
        e.to_string()
      ))),
    }
  }

  #[cfg(feature = "json")]
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
        #[cfg(feature = "raw")]
        Success::Serialized(v) => {
          raw::raw_deserialize::<T>(v).map_err(|e| Error::DeserializationError(e.to_string()))
        }
        #[cfg(feature = "json")]
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
}

impl From<Packet> for MessageTransport {
  fn from(output: Packet) -> MessageTransport {
    match output {
      Packet::V0(v) => match v {
        v0::Payload::Exception(v) => MessageTransport::Failure(Failure::Exception(v)),
        v0::Payload::Error(v) => MessageTransport::Failure(Failure::Error(v)),
        v0::Payload::Invalid => MessageTransport::Failure(Failure::Invalid),
        v0::Payload::MessagePack(bytes) => MessageTransport::Success(Success::MessagePack(bytes)),
        #[cfg(feature = "json")]
        v0::Payload::Json(v) => MessageTransport::Success(Success::Json(v)),
        #[cfg(not(feature = "json"))]
        v0::Payload::Json(v) => MessageTransport::success(&v),
        #[cfg(feature = "raw")]
        v0::Payload::Success(v) => MessageTransport::Success(Success::Serialized(v)),
        #[cfg(not(feature = "raw"))]
        v0::Payload::Success(v) => MessageTransport::success(&v),
        v0::Payload::Done => MessageTransport::Signal(MessageSignal::Done),
        v0::Payload::OpenBracket => MessageTransport::Signal(MessageSignal::OpenBracket),
        v0::Payload::CloseBracket => MessageTransport::Signal(MessageSignal::CloseBracket),
      },
      Packet::V1(v) => match v {
        vino_packet::v1::Payload::Success(success) => match success {
          vino_packet::v1::Success::MessagePack(bytes) => {
            MessageTransport::Success(Success::MessagePack(bytes))
          }
          #[cfg(feature = "raw")]
          vino_packet::v1::Success::Success(v) => MessageTransport::Success(Success::Serialized(v)),
          #[cfg(not(feature = "raw"))]
          vino_packet::v1::Success::Success(v) => MessageTransport::success(&v),
          #[cfg(feature = "json")]
          vino_packet::v1::Success::Json(v) => MessageTransport::Success(Success::Json(v)),
          #[cfg(not(feature = "json"))]
          vino_packet::v1::Success::Json(v) => MessageTransport::success(&v),
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
        #[cfg(feature = "raw")]
        Success::Serialized(v) => Packet::V1(v1::Payload::Success(v1::Success::Success(v))),
        #[cfg(feature = "json")]
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
    match self {
      Failure::Invalid => f.write_str("Invalid"),
      Failure::Exception(v) => f.write_fmt(format_args!("Exception: {}", v)),
      Failure::Error(v) => f.write_fmt(format_args!("Exception: {}", v)),
    }
  }
}
impl Display for Success {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "{}",
      match self {
        Success::MessagePack(_) => "MessagePack",
        #[cfg(feature = "raw")]
        Success::Serialized(_) => "Success",
        #[cfg(feature = "json")]
        Success::Json(_) => "JSON",
      }
    ))
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  #[test_env_log::test]
  #[cfg(feature = "json")]
  fn serializes_done() -> Result<()> {
    let close = MessageTransport::done();
    let value = close.into_json();
    println!("Value: {}", value);
    assert_eq!(value.to_string(), r#"{"signal":"Done","value":null}"#);
    Ok(())
  }

  #[test_env_log::test]
  fn messagepack_rt() -> Result<()> {
    // let mut original = TransportMap::new();
    let mut payload = MessageTransport::success(&false);
    println!("payload: {:?}", payload);
    payload.to_messagepack();
    let result: bool = payload.try_into()?;
    assert_eq!(result, false);
    Ok(())
  }
}
