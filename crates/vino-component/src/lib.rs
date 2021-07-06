pub mod v0;
pub mod packet {
  pub use crate::v0;
}
pub mod claims;
// pub use claims::ComponentClaims;
use serde::{
  Deserialize,
  Serialize,
};
use v0::Payload;
use vino_codec::messagepack::deserialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// The output payload that component's push out of output ports
pub enum Packet {
  /// Version 0 of the payload format (unstable)
  #[serde(rename = "0")]
  V0(v0::Payload),
}

#[derive(Debug)]
pub enum Error {
  Invalid,
  Exception(String),
  Error(String),
  InternalError(vino_codec::Error),
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::Invalid => write!(f, "Invalid"),
      Error::Exception(v) => write!(f, "{}", v),
      Error::Error(v) => write!(f, "{}", v),
      Error::InternalError(e) => write!(f, "{}", e.to_string()),
    }
  }
}

impl std::error::Error for Error {}

impl Packet {
  pub fn try_into<'de, T: Deserialize<'de>>(self) -> Result<T, Error> {
    match self {
      Packet::V0(v) => match v {
        v0::Payload::Invalid => Err(Error::Invalid),
        v0::Payload::Exception(v) => Err(Error::Exception(v)),
        v0::Payload::Error(v) => Err(Error::Error(v)),
        v0::Payload::MessagePack(buf) => deserialize::<T>(&buf).map_err(Error::InternalError),
        v0::Payload::Close => todo!(),
        v0::Payload::OpenBracket => todo!(),
        v0::Payload::CloseBracket => todo!(),
      },
    }
  }
}

impl From<&Vec<u8>> for Packet {
  fn from(buf: &Vec<u8>) -> Self {
    match deserialize::<Packet>(buf) {
      Ok(packet) => packet,
      Err(e) => Packet::V0(Payload::Error(format!("Error deserializing packet: {}", e))),
    }
  }
}

impl From<&[u8]> for Packet {
  fn from(buf: &[u8]) -> Self {
    match deserialize::<Packet>(buf) {
      Ok(packet) => packet,
      Err(e) => Packet::V0(Payload::Error(format!("Error deserializing packet: {}", e))),
    }
  }
}
