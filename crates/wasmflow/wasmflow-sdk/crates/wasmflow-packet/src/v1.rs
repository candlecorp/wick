use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use wasmflow_codec::messagepack::rmp_serialize;
use wasmflow_codec::raw::raw_serialize;

use crate::error::Error;
use crate::Packet as RootPacket;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[must_use]
/// A component's output data.
pub enum Packet {
  /// A successful message.
  #[serde(rename = "0")]
  Success(Serialized),

  /// A message stemming from an error somewhere.
  #[serde(rename = "1")]
  Failure(Failure),

  /// An error. Used by library authors to indicate a problem.
  #[serde(rename = "2")]
  Signal(Signal),
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

/// A success message.
#[must_use]
pub enum Serialized {
  /// A message carrying a payload encoded with MessagePack.
  #[serde(rename = "0")]
  MessagePack(Vec<u8>),

  /// A successful payload in a generic intermediary format.
  #[serde(rename = "1")]
  Struct(serde_value::Value),

  /// A payload represented as a raw JSON String.
  #[serde(rename = "2")]
  Json(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// A Failure message.
#[must_use]
pub enum Failure {
  /// Invalid payload. Used when a default message is unavoidable.
  #[serde(rename = "0")]
  Invalid,

  /// A message carrying an exception (an error that short-circuited a port's downstream).
  #[serde(rename = "1")]
  Exception(String),

  /// A message carrying an error (an error that short circuited all downstreams from a component).
  #[serde(rename = "2")]
  Error(String),
}

#[allow(missing_copy_implementations)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Internal signals that need to be handled before propagating to a downstream consumer.
#[must_use]
pub enum Signal {
  /// Indicates the job that opened this port is finished with it.
  #[serde(rename = "0")]
  Done,

  /// Indicates that a message is coming down in chunks and this is the start.
  #[doc(hidden)]
  #[serde(rename = "1")]
  OpenBracket,

  /// Indicates a chunked message has been completed.
  #[serde(rename = "2")]
  #[doc(hidden)]
  CloseBracket,
}

impl Packet {
  /// A one-liner to turn a serializable object into a [Serialized::MessagePack] variant.
  pub fn messagepack<T: Serialize>(t: &T) -> Self {
    match rmp_serialize(t) {
      Ok(bytes) => Self::Success(Serialized::MessagePack(bytes)),
      Err(e) => Self::Failure(Failure::Error(e.to_string())),
    }
  }

  /// A one-liner to turn a serializable object into a [Packet::Success] variant.
  pub fn success<T: Serialize>(t: &T) -> Self {
    match raw_serialize(t) {
      Ok(bytes) => Self::Success(Serialized::Struct(bytes)),
      Err(e) => Self::Failure(Failure::Error(e.to_string())),
    }
  }

  /// Creates a [Packet::Signal(Signal::Done)]
  pub fn done() -> Self {
    Self::Signal(Signal::Done)
  }

  /// Creates a [Packet::Failure(Failure::Exception)]
  pub fn exception<T: AsRef<str>>(msg: T) -> Self {
    Self::Failure(Failure::Exception(msg.as_ref().to_owned()))
  }

  /// Creates a [Packet::Failure(Failure::Error)]
  pub fn error<T: AsRef<str>>(msg: T) -> Self {
    Self::Failure(Failure::Error(msg.as_ref().to_owned()))
  }

  /// Try to deserialize a [Packet] into the target type
  pub fn deserialize<T: DeserializeOwned>(self) -> Result<T, Error> {
    try_from(self)
  }
}

fn try_from<T: DeserializeOwned>(value: Packet) -> Result<T, Error> {
  match value {
    Packet::Success(success) => match success {
      Serialized::MessagePack(v) => wasmflow_codec::messagepack::deserialize(&v).map_err(Error::DeserializationError),
      Serialized::Struct(v) => wasmflow_codec::raw::deserialize(v).map_err(Error::DeserializationError),
      Serialized::Json(v) => wasmflow_codec::json::deserialize(&v).map_err(Error::DeserializationError),
    },
    Packet::Failure(failure) => match failure {
      Failure::Invalid => Err(Error::Invalid),
      Failure::Exception(v) => Err(Error::Exception(v)),
      Failure::Error(v) => Err(Error::Error(v)),
    },
    Packet::Signal(_) => Err(Error::Signal),
  }
}

impl From<Packet> for RootPacket {
  fn from(v: Packet) -> Self {
    Self::V1(v)
  }
}

impl From<Serialized> for Packet {
  fn from(v: Serialized) -> Self {
    Packet::Success(v)
  }
}

impl From<Failure> for Packet {
  fn from(v: Failure) -> Self {
    Packet::Failure(v)
  }
}

impl From<Signal> for Packet {
  fn from(v: Signal) -> Self {
    Packet::Signal(v)
  }
}

#[cfg(feature = "v0")]
impl From<super::v0::Payload> for Packet {
  fn from(p: super::v0::Payload) -> Self {
    match p {
      crate::v0::Payload::Invalid => Packet::Failure(Failure::Invalid),
      crate::v0::Payload::Exception(v) => Packet::Failure(Failure::Exception(v)),
      crate::v0::Payload::Error(v) => Packet::Failure(Failure::Error(v)),
      crate::v0::Payload::MessagePack(v) => Packet::Success(Serialized::MessagePack(v)),
      crate::v0::Payload::Done => Packet::Signal(Signal::Done),
      crate::v0::Payload::OpenBracket => Packet::Signal(Signal::OpenBracket),
      crate::v0::Payload::CloseBracket => Packet::Signal(Signal::CloseBracket),
      crate::v0::Payload::Success(v) => Packet::Success(Serialized::Struct(v)),
      crate::v0::Payload::Json(v) => Packet::Success(Serialized::Json(v)),
    }
  }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
/// A map of port names to packets.
pub struct PacketMap {
  inner: HashMap<String, Packet>,
}

impl PacketMap {
  /// Create a new [PacketMap]
  #[must_use]
  pub fn new(map: HashMap<String, Packet>) -> Self {
    Self { inner: map }
  }

  /// Remove a [Packet] from the [PacketMap].
  #[must_use]
  pub fn remove(&mut self, port: &str) -> Option<Packet> {
    self.inner.remove(port)
  }

  /// Insert a [Packet] from the [PacketMap].
  pub fn insert(&mut self, port: String, value: Packet) {
    self.inner.insert(port, value);
  }
}

impl IntoIterator for PacketMap {
  type Item = (String, Packet);
  type IntoIter = std::collections::hash_map::IntoIter<String, Packet>;

  fn into_iter(self) -> Self::IntoIter {
    self.inner.into_iter()
  }
}
