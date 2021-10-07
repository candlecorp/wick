use serde::{
  Deserialize,
  Serialize,
};
use vino_codec::messagepack;

pub use crate::{
  v0,
  v1,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// The output payload that component's push out of output ports.
pub enum Packet {
  /// Version 0 of the payload format (unstable).
  #[serde(rename = "0")]
  V0(v0::Payload),
  /// Version 1 of the payload format (alpha).
  #[serde(rename = "0")]
  V1(v1::Payload),
}

impl Packet {
  #[must_use]
  /// Does the [Packet] signify the originating job is completed?.
  pub fn is_done(&self) -> bool {
    match self {
      Packet::V0(v) => matches!(v, v0::Payload::Done | v0::Payload::Error(_)),
      Packet::V1(v) => matches!(
        v,
        v1::Payload::Signal(v1::Signal::Done) | v1::Payload::Failure(v1::Failure::Error(_))
      ),
    }
  }
}

impl From<&Vec<u8>> for Packet {
  fn from(buf: &Vec<u8>) -> Self {
    match messagepack::deserialize::<Packet>(buf) {
      Ok(packet) => packet,
      Err(e) => Packet::V0(v0::Payload::Error(format!(
        "Error deserializing packet: {}",
        e
      ))),
    }
  }
}

impl From<&[u8]> for Packet {
  fn from(buf: &[u8]) -> Self {
    match messagepack::deserialize::<Packet>(buf) {
      Ok(packet) => packet,
      Err(e) => Packet::V0(v0::Payload::Error(format!(
        "Error deserializing packet: {}",
        e
      ))),
    }
  }
}

#[derive(Debug, Clone)]
/// A [PacketWrapper] is a wrapper around a [Packet] with the port name embedded.
pub struct PacketWrapper {
  /// The port name.
  pub port: String,
  /// The wrapped packet [Packet].
  pub payload: Packet,
}
