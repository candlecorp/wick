use std::collections::HashMap;

use serde::{
  Deserialize,
  Serialize,
};
use vino_component::{
  v0,
  Output,
};

use crate::{
  Error,
  Result,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageTransport {
  MessagePack(Vec<u8>),
  MultiBytes(HashMap<String, Vec<u8>>),
  Exception(String),
  Error(String),
  Test(String),
  Invalid,
}

impl Default for MessageTransport {
  fn default() -> Self {
    Self::Invalid
  }
}

impl MessageTransport {
  pub fn into_bytes(self) -> Result<Vec<u8>> {
    match self {
      MessageTransport::MessagePack(v) => Ok(v),
      _ => Err(Error::PayloadConversionError("Invalid payload".to_string())),
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

impl From<Output> for MessageTransport {
  fn from(output: Output) -> MessageTransport {
    match output {
      Output::V0(v) => match v {
        v0::Payload::Exception(v) => MessageTransport::Exception(v),
        v0::Payload::Error(v) => MessageTransport::Error(v),
        v0::Payload::Invalid => MessageTransport::Invalid,
        v0::Payload::MessagePack(bytes) => MessageTransport::MessagePack(bytes),
      },
    }
  }
}
