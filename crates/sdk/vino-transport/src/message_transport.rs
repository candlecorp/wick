use std::collections::HashMap;

use serde::{
  Deserialize,
  Serialize,
};
use vino_component::{
  v0,
  Packet,
};

use crate::{
  Error,
  Result,
};

pub type PortMap = HashMap<String, MessageTransport>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageTransport {
  Invalid,
  Exception(String),
  Error(String),
  MessagePack(Vec<u8>),
  MultiBytes(HashMap<String, Vec<u8>>),
  OutputMap(PortMap),
  Test(String),
}

impl Default for MessageTransport {
  fn default() -> Self {
    Self::Invalid
  }
}

impl MessageTransport {
  pub fn is_ok(&self) -> bool {
    match self {
      MessageTransport::MessagePack(_) => true,
      MessageTransport::MultiBytes(_) => true,
      MessageTransport::Test(_) => true,
      MessageTransport::Exception(_) => false,
      MessageTransport::Error(_) => false,
      MessageTransport::Invalid => false,
      MessageTransport::OutputMap(_) => true,
    }
  }
  pub fn into_bytes(self) -> Result<Vec<u8>> {
    match self {
      MessageTransport::MessagePack(v) => Ok(v),
      _ => Err(Error::PayloadConversionError("Invalid payload".to_string())),
    }
  }
  pub fn into_output_map(self) -> Result<PortMap> {
    match self {
      MessageTransport::OutputMap(v) => Ok(v),
      _ => Err(Error::PayloadConversionError(
        "Invalid payload, not an output map".to_string(),
      )),
    }
  }
  pub fn into_multibytes(self) -> Result<HashMap<String, Vec<u8>>> {
    match self {
      MessageTransport::MultiBytes(v) => Ok(v),
      _ => Err(Error::PayloadConversionError("Invalid payload".to_string())),
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
      },
    }
  }
}
