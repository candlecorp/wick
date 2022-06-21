use std::collections::HashMap;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{Error, Failure, MessageSignal, MessageTransport, Serialized};
/// A simplified JSON representation of a MessageTransport
#[derive(Debug, Clone, Eq, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct TransportJson {
  /// Error message for the port if it exists.
  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error_msg: Option<String>,

  /// The error kind if it exists.
  #[serde(default)]
  #[serde(skip_serializing_if = "JsonError::is_none")]
  pub error_kind: JsonError,

  /// The Signal if the message was a [MessageTransport::Signal]
  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub signal: Option<MessageSignal>,

  /// The return value.
  pub value: serde_json::Value,
}

impl From<TransportJson> for MessageTransport {
  fn from(v: TransportJson) -> Self {
    match v.error_kind {
      JsonError::None => match v.signal {
        Some(signal) => MessageTransport::Signal(signal),
        None => {
          // We just parsed JSON and are now turning part of it
          // back into JSON which doesn't feel good. This is only
          // used for command line testing and piping but if it ends
          // up being used for more it will need to be better handled.
          MessageTransport::Success(Serialized::Json(v.value.to_string()))
        }
      },
      JsonError::Exception => match v.error_msg {
        Some(err) => MessageTransport::Failure(Failure::Exception(err)),
        None => MessageTransport::Failure(Failure::Exception("<No message passed with exception>".to_owned())),
      },
      JsonError::Error => match v.error_msg {
        Some(err) => MessageTransport::Failure(Failure::Error(err)),
        None => MessageTransport::Failure(Failure::Error("<No message passed with exception>".to_owned())),
      },
      JsonError::InternalError => MessageTransport::Failure(Failure::Error("Internal Error (10001)".to_owned())),
    }
  }
}

/// The kinds of errors that a [TransportJson] can carry
#[derive(Debug, Clone, Copy, Eq, Serialize, Deserialize, PartialEq)]
#[must_use]
pub enum JsonError {
  /// No error
  None,

  /// A message from a [Failure::Exception]
  Exception,

  /// A message from a [Failure::Error]
  Error,

  /// An error originating internally
  InternalError,
}

impl Default for JsonError {
  fn default() -> Self {
    JsonError::None
  }
}

impl Display for JsonError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      JsonError::None => "None",
      JsonError::Exception => "Exception",
      JsonError::Error => "Error",
      JsonError::InternalError => "Internal Error",
    };
    f.write_str(s)
  }
}

impl JsonError {
  #[must_use]
  /// This is analogous to Option::is_none for a [JsonError] kind
  pub fn is_none(&self) -> bool {
    matches!(self, JsonError::None)
  }
  #[must_use]
  /// This is analogous to Option::is_some for a [JsonError] kind
  pub fn is_some(&self) -> bool {
    !matches!(self, JsonError::None)
  }
}

fn handle_result_conversion(result: Result<serde_json::Value, String>) -> TransportJson {
  match result {
    Ok(payload) => TransportJson {
      value: payload,
      signal: None,
      error_msg: None,
      error_kind: JsonError::None,
    },
    Err(e) => {
      let msg = format!("Error deserializing messagepack payload to JSON value: {:?}", e);
      error!("{}", msg);
      TransportJson {
        value: serde_json::value::Value::Null,
        signal: None,
        error_msg: Some(msg),
        error_kind: JsonError::InternalError,
      }
    }
  }
}

impl MessageTransport {
  /// Converts a [MessageTransport] into [serde_json::Value]
  /// representation of a [TransportJson]
  #[must_use]
  pub fn as_json(&self) -> serde_json::Value {
    let output = match self {
      MessageTransport::Success(success) => match success {
        Serialized::MessagePack(bytes) => handle_result_conversion(
          wasmflow_codec::messagepack::deserialize::<serde_json::Value>(bytes).map_err(|e| e.to_string()),
        ),
        Serialized::Struct(v) => handle_result_conversion(
          wasmflow_codec::raw::deserialize::<serde_json::Value>(v.clone()).map_err(|e| e.to_string()),
        ),
        Serialized::Json(v) => {
          handle_result_conversion(wasmflow_codec::json::deserialize::<serde_json::Value>(v).map_err(|e| e.to_string()))
        }
      },
      MessageTransport::Failure(failure) => match &failure {
        Failure::Invalid => TransportJson {
          value: serde_json::value::Value::Null,
          signal: None,
          error_msg: Some("Invalid value".to_owned()),
          error_kind: JsonError::Error,
        },
        Failure::Exception(v) => TransportJson {
          value: serde_json::value::Value::Null,
          signal: None,
          error_msg: Some(v.clone()),
          error_kind: JsonError::Exception,
        },
        Failure::Error(v) => TransportJson {
          value: serde_json::value::Value::Null,
          signal: None,
          error_msg: Some(v.clone()),
          error_kind: JsonError::Error,
        },
      },
      MessageTransport::Signal(s) => TransportJson {
        value: serde_json::value::Value::Null,
        signal: Some(s.clone()),
        error_msg: None,
        error_kind: JsonError::None,
      },
    };

    serde_json::to_value(&output).unwrap_or_else(|_| {
      let error = TransportJson {
        value: serde_json::value::Value::Null,
        signal: None,
        error_msg: Some("Error serializing packet into JSON.".to_owned()),
        error_kind: JsonError::InternalError,
      };
      serde_json::to_value(&error).unwrap()
    })
  }

  /// Convert a [HashMap<String, MessageTransport>] into a [serde_json::value::Map]
  #[must_use]
  pub fn map_to_json(
    raw_result: HashMap<String, MessageTransport>,
  ) -> serde_json::value::Map<String, serde_json::Value> {
    raw_result
      .into_iter()
      .map(|(k, payload)| {
        (
          k,
          payload.deserialize().unwrap_or_else(|e: Error| {
            serde_json::json!({ "error": format!("Internal error: {:?}, invalid format", e.to_string()) })
          }),
        )
      })
      .collect()
  }
}
