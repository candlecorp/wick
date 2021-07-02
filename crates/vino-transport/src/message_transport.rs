use std::collections::HashMap;
use std::fmt::Display;

use log::error;
use serde::{
  Deserialize,
  Serialize,
};
use vino_codec::messagepack::deserialize;
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
  Signal(MessageSignal),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageSignal {
  Close,
  OpenBracket,
  CloseBracket,
}

impl Default for MessageTransport {
  fn default() -> Self {
    Self::Invalid
  }
}

pub struct JsonOutput {
  error_msg: Option<String>,
  error_type: JsonError,
  value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JsonError {
  None,
  Exception,
  Error,
  InternalError,
}

impl Display for JsonError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      JsonError::None => "None",
      JsonError::Exception => "Exception",
      JsonError::Error => "Error",
      JsonError::InternalError => "InternalError",
    };
    f.write_str(s)
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
      MessageTransport::Signal(_) => false,
    }
  }
  pub fn into_json(self) -> serde_json::Value {
    let output = match self {
      MessageTransport::Invalid => JsonOutput {
        value: serde_json::value::Value::Null,
        error_msg: Some("Invalid value".to_owned()),
        error_type: JsonError::Error,
      },
      MessageTransport::Exception(v) => JsonOutput {
        value: serde_json::value::Value::Null,
        error_msg: Some(v),
        error_type: JsonError::Exception,
      },
      MessageTransport::Error(v) => JsonOutput {
        value: serde_json::value::Value::Null,
        error_msg: Some(v),
        error_type: JsonError::Error,
      },
      MessageTransport::MessagePack(bytes) => match deserialize::<serde_json::Value>(&bytes) {
        Ok(payload) => JsonOutput {
          value: payload,
          error_msg: None,
          error_type: JsonError::None,
        },
        Err(e) => {
          let msg = format!(
            "Error deserializing messagepack payload to JSON value: {:?}",
            e
          );
          error!("{}", msg);
          JsonOutput {
            value: serde_json::value::Value::Null,
            error_msg: Some(msg),
            error_type: JsonError::InternalError,
          }
        }
      },
      _ => {
        error!(
          "Unhandled internal message trying to convert to JSON: {:?}",
          self
        );
        JsonOutput {
          value: serde_json::value::Value::Null,
          error_msg: Some("Unhandled internal message".to_owned()),
          error_type: JsonError::InternalError,
        }
      }
    };
    let mut map = serde_json::Map::new();
    map.insert("value".to_owned(), output.value);
    if let Some(msg) = output.error_msg {
      map.insert("error_msg".to_owned(), serde_json::Value::String(msg));
    }
    map.insert(
      "error_kind".to_owned(),
      serde_json::Value::String(output.error_type.to_string()),
    );
    serde_json::value::Value::Object(map)
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
        v0::Payload::Close => MessageTransport::Signal(MessageSignal::Close),
        v0::Payload::OpenBracket => MessageTransport::Signal(MessageSignal::OpenBracket),
        v0::Payload::CloseBracket => MessageTransport::Signal(MessageSignal::CloseBracket),
      },
    }
  }
}
