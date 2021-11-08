use serde::{Deserialize, Serialize};
use vino_packet::Packet;
use vino_transport::MessageTransport;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[must_use]
#[serde(transparent)]
/// Raw value type
pub struct RawPacket(MessageTransport);

impl RawPacket {
  /// Constructor for an empty Raw value
  pub fn new(message: MessageTransport) -> Self {
    Self(message)
  }

  /// Returns the contained byte array
  pub fn into_inner(self) -> MessageTransport {
    self.0
  }
}
impl From<MessageTransport> for RawPacket {
  fn from(v: MessageTransport) -> Self {
    RawPacket(v)
  }
}
impl From<RawPacket> for MessageTransport {
  fn from(v: RawPacket) -> Self {
    v.0
  }
}

impl From<RawPacket> for vino_packet::v1::Payload {
  fn from(v: RawPacket) -> Self {
    let packet: Packet = v.into_inner().into();
    match packet {
      Packet::V0(v) => v.into(),
      Packet::V1(v) => v,
    }
  }
}
