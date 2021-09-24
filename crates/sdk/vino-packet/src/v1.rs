use serde::{
  Deserialize,
  Serialize,
};
use vino_codec::messagepack::rmp_serialize;
use vino_codec::raw::raw_serialize;

use crate::Packet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[must_use]
/// A component's output data.
pub enum Payload {
  /// TODO
  #[serde(rename = "0")]
  Success(Success),

  /// TODO
  #[serde(rename = "1")]
  Failure(Failure),

  /// An error. Used by library authors to indicate a problem.
  #[serde(rename = "2")]
  Signal(Signal),
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

/// TODO
#[must_use]
pub enum Success {
  /// TODO
  #[serde(rename = "0")]
  MessagePack(Vec<u8>),

  /// TODO
  #[serde(rename = "1")]
  Success(serde_value::Value),

  /// TODO
  #[serde(rename = "2")]
  Json(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// TODO
#[must_use]
pub enum Failure {
  /// Invalid payload. Used when a default is needed.
  #[serde(rename = "0")]
  Invalid,

  /// An exception. Used by application authors to indicate a use case exception or user error.
  #[serde(rename = "1")]
  Exception(String),

  /// An error. Used by library authors to indicate a problem.
  #[serde(rename = "2")]
  Error(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
/// TODO
#[must_use]
pub enum Signal {
  /// A message that signifies the port is done.
  #[serde(rename = "0")]
  Done,

  /// A message that signals the start of bracketed data (think of it like an opening bracket '[').
  #[doc(hidden)]
  #[serde(rename = "1")]
  OpenBracket,

  /// A message that signals the end of bracketed data (think of it like an closing bracket ']').
  #[serde(rename = "2")]
  #[doc(hidden)]
  CloseBracket,
}

impl Payload {
  /// A one-liner to turn a serializable object into a [Payload::MessagePack] variant.
  pub fn messagepack<T: Serialize>(t: &T) -> Self {
    match rmp_serialize(t) {
      Ok(bytes) => Self::Success(Success::MessagePack(bytes)),
      Err(e) => Self::Failure(Failure::Error(e.to_string())),
    }
  }

  /// A one-liner to turn a serializable object into a [Payload::Success] variant.
  pub fn success<T: Serialize>(t: &T) -> Self {
    match raw_serialize(t) {
      Ok(bytes) => Self::Success(Success::Success(bytes)),
      Err(e) => Self::Failure(Failure::Error(e.to_string())),
    }
  }

  /// TODO
  pub fn done() -> Self {
    Self::Signal(Signal::Done)
  }

  /// TODO
  pub fn exception<T: AsRef<str>>(msg: T) -> Self {
    Self::Failure(Failure::Exception(msg.as_ref().to_owned()))
  }

  /// TODO
  pub fn error<T: AsRef<str>>(msg: T) -> Self {
    Self::Failure(Failure::Error(msg.as_ref().to_owned()))
  }
}

impl From<Payload> for Packet {
  fn from(v: Payload) -> Self {
    Packet::V1(v)
  }
}

impl From<Success> for Payload {
  fn from(v: Success) -> Self {
    Payload::Success(v)
  }
}

impl From<Failure> for Payload {
  fn from(v: Failure) -> Self {
    Payload::Failure(v)
  }
}

impl From<Signal> for Payload {
  fn from(v: Signal) -> Self {
    Payload::Signal(v)
  }
}

impl From<super::v0::Payload> for Payload {
  fn from(p: super::v0::Payload) -> Self {
    match p {
      crate::v0::Payload::Invalid => Payload::Failure(Failure::Invalid),
      crate::v0::Payload::Exception(v) => Payload::Failure(Failure::Exception(v)),
      crate::v0::Payload::Error(v) => Payload::Failure(Failure::Error(v)),
      crate::v0::Payload::MessagePack(v) => Payload::Success(Success::MessagePack(v)),
      crate::v0::Payload::Done => Payload::Signal(Signal::Done),
      crate::v0::Payload::OpenBracket => Payload::Signal(Signal::OpenBracket),
      crate::v0::Payload::CloseBracket => Payload::Signal(Signal::CloseBracket),
      crate::v0::Payload::Success(v) => Payload::Success(Success::Success(v)),
      crate::v0::Payload::Json(v) => Payload::Success(Success::Json(v)),
    }
  }
}
