use serde::de::DeserializeOwned;
use serde::{
  Deserialize,
  Serialize,
};
use vino_packet::PacketWrapper;

use crate::error::TransportError;
use crate::{
  MessageSignal,
  MessageTransport,
  SYSTEM_ID,
};
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
  pub fn new(port: &str, payload: MessageTransport) -> Self {
    Self {
      port: port.to_owned(),
      payload,
    }
  }

  /// Constructs a [TransportWrapper] that represents a close message.
  pub fn new_system_close() -> Self {
    Self::new(SYSTEM_ID, MessageTransport::Signal(MessageSignal::Done))
  }

  /// Returns true if the [TransportWrapper] is a system message with the payload of [MessageSignal::Done].
  #[must_use]
  pub fn is_system_close(&self) -> bool {
    self.port == SYSTEM_ID && self.payload == MessageTransport::done()
  }

  /// Constructor for a [TransportWrapper] with a port of [crate::COMPONENT_ERROR] indicating an internal error occurred.
  pub fn internal_error(payload: MessageTransport) -> Self {
    Self {
      port: crate::COMPONENT_ERROR.to_owned(),
      payload,
    }
  }

  /// Attempt to deserialize the contained [MessageTransport] into the destination value.
  pub fn try_into<T: DeserializeOwned>(self) -> Result<T> {
    self.payload.try_into()
  }

  /// Converts the embedded [MessageTransport] into a [serde_json::Value::Object]
  /// map of port names to [TransportJson]s
  #[must_use]
  #[cfg(feature = "json")]
  pub fn into_json(self) -> serde_json::Value {
    let payload = self.payload.into_json();

    let mut map = serde_json::Map::new();
    map.insert(self.port, payload);

    serde_json::value::Value::Object(map)
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
