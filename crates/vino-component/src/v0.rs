use serde::ser::{
  Error,
  SerializeSeq,
};
use serde::{
  Deserialize,
  Serialize,
  Serializer,
};
use vino_codec::messagepack::rmp_serialize;

#[allow(clippy::needless_lifetimes)]
fn to_messagepack<'a, S, T>(data: &'a T, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
  T: Serialize,
{
  let bytes = rmp_serialize(data).map_err(|e| S::Error::custom(e.to_string()))?;
  let mut seq = s.serialize_seq(Some(bytes.len()))?;
  for byte in bytes {
    seq.serialize_element(&byte)?
  }
  seq.end()
  // s.serialize_bytes(&bytes)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// A component's output data (version 0: unstable)
pub enum Payload<T: Serialize> {
  /// Invalid payload. Used when a default is needed.
  #[serde(rename = "0")]
  Invalid,
  /// A success payload that can be serialized to MessagePack for transport.
  #[serde(rename(serialize = "3"))]
  #[serde(serialize_with = "to_messagepack")]
  Serializable(T),
  /// An exception. Used by application authors to indicate a use case exception or user error.
  #[serde(rename = "1")]
  Exception(String),
  /// An error. Used by library authors to indicate a problem.
  #[serde(rename = "2")]
  Error(String),
  /// A MessagePack payload. Meant to pass through untouched until reaching its final destination, most commonly used with WaPC WebAssembly components.
  #[serde(rename = "3")]
  MessagePack(Vec<u8>),
}
