use serde::{
  Deserialize,
  Serialize,
};
use vino_codec::messagepack::rmp_serialize;
use vino_codec::raw::raw_serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// A component's output data (version 0: unstable)
pub enum Payload {
  /// Invalid payload. Used when a default is needed.
  #[serde(rename = "0")]
  Invalid,
  /// An exception. Used by application authors to indicate a use case exception or user error.
  #[serde(rename = "1")]
  Exception(String),
  /// An error. Used by library authors to indicate a problem.
  #[serde(rename = "2")]
  Error(String),
  /// A MessagePack payload that has not yet been deserialized
  #[serde(rename = "3")]
  MessagePack(Vec<u8>),
  /// A message that signifies the port is closed.
  #[serde(rename = "4")]
  Close,
  /// A message that signals the start of bracketed data (think of it like an opening bracket '[')
  #[serde(rename = "5")]
  OpenBracket,
  /// A message that signals the end of bracketed data (think of it like an closing bracket ']')
  #[serde(rename = "6")]
  CloseBracket,
  /// A successful payload
  #[serde(rename = "7")]
  Success(serde_value::Value),
  /// A JSON payload that has not yet been deserialized.
  #[serde(rename = "8")]
  Json(String),
}

impl Payload {
  /// A one-liner to turn a serializable object into a [Payload::MessagePack] variant
  pub fn to_messagepack<T: Serialize>(t: &T) -> Self {
    match rmp_serialize(t) {
      Ok(bytes) => Self::MessagePack(bytes),
      Err(e) => Self::Error(e.to_string()),
    }
  }
  /// A one-liner to turn a serializable object into a [Payload::Success] variant
  pub fn success<T: Serialize>(t: &T) -> Self {
    match raw_serialize(t) {
      Ok(value) => Self::Success(value),
      Err(e) => Self::Error(e.to_string()),
    }
  }
}
