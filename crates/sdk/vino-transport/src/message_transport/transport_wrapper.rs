use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use vino_packet::PacketWrapper;

use crate::error::TransportError;
use crate::{MessageTransport, SYSTEM_ID};
pub(crate) type Result<T> = std::result::Result<T, TransportError>;

/// A [TransportWrapper] is a wrapper around a [MessageTransport] with the port name it originated from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[must_use]
pub struct TransportWrapper {
  /// The port the message originated from.
  pub port: String,
  /// The port's output.
  pub payload: MessageTransport,
}

impl TransportWrapper {
  /// Constructor for [TransportWrapper]s.
  pub fn new<T: AsRef<str>>(port: T, payload: MessageTransport) -> Self {
    Self {
      port: port.as_ref().to_owned(),
      payload,
    }
  }

  /// Constructs a [TransportWrapper] that represents a close message.
  pub fn done<T: AsRef<str>>(port: T) -> Self {
    Self::new(port, MessageTransport::done())
  }

  /// Constructs a [TransportWrapper] that represents a close message.
  pub fn new_system_close() -> Self {
    Self::new(SYSTEM_ID, MessageTransport::done())
  }

  /// Returns true if the [TransportWrapper] is a system message with the payload of [MessageSignal::Done].
  #[must_use]
  pub fn is_system_close(&self) -> bool {
    self.port == SYSTEM_ID && self.payload == MessageTransport::done()
  }

  /// Returns true if the [TransportWrapper] originated from a component-wide error.
  #[must_use]
  pub fn is_component_error(&self) -> bool {
    self.port == crate::COMPONENT_ERROR
  }

  /// Returns Some(&str) if the [TransportWrapper] contains an error, None otherwise.
  #[must_use]
  pub fn error(&self) -> Option<&str> {
    match &self.payload {
      MessageTransport::Failure(f) => Some(f.message()),
      _ => None,
    }
  }

  /// Constructor for a [TransportWrapper] with a port of [crate::COMPONENT_ERROR] indicating an internal error occurred.
  pub fn component_error(payload: MessageTransport) -> Self {
    Self {
      port: crate::COMPONENT_ERROR.to_owned(),
      payload,
    }
  }

  /// Attempt to deserialize the contained [MessageTransport] into the destination value.
  pub fn deserialize<T: DeserializeOwned>(self) -> Result<T> {
    self.payload.deserialize()
  }

  /// Converts the embedded [MessageTransport] into a [serde_json::Value::Object]
  /// map of port names to [TransportJson]s
  #[must_use]
  #[cfg(feature = "json")]
  pub fn as_json(&self) -> serde_json::Value {
    let payload = self.payload.as_json();

    let mut map = serde_json::Map::new();
    map.insert(self.port.clone(), payload);

    serde_json::value::Value::Object(map)
  }
}

#[cfg(feature = "json")]
impl std::fmt::Display for TransportWrapper {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_json())
  }
}

impl From<PacketWrapper> for TransportWrapper {
  fn from(p: PacketWrapper) -> Self {
    Self {
      port: p.port,
      payload: p.payload.into(),
    }
  }
}

impl From<(String, MessageTransport)> for TransportWrapper {
  fn from(entry: (String, MessageTransport)) -> Self {
    Self {
      payload: entry.1,
      port: entry.0,
    }
  }
}
