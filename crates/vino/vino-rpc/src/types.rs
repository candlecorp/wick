/// Conversion utilities for RPC data structures.
pub(crate) mod conversions;
use std::time::Duration;

pub use vino_types::*;

pub use conversions::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use vino_packet::Packet;
use vino_transport::{Failure, MessageTransport, TransportWrapper};

use crate::error::RpcError;
use crate::rpc::{message_kind, MessageKind, Output};
use crate::Result;

/// Important statistics for the hosted components.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Statistics {
  /// The name of the component.
  pub name: String,
  /// The number of times a component has been called.
  pub runs: u32,
  /// The number of times the component resulted in an unrecoverable error.
  pub errors: u32,
  /// Execution duration statistics.
  pub execution_duration: Option<DurationStatistics>,
}

mod as_micros {
  use std::convert::TryInto;
  use std::time::Duration;

  use serde::{Deserialize, Deserializer, Serializer};

  pub(crate) fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_u64(duration.as_micros().try_into().unwrap_or(u64::MAX))
  }
  pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
  where
    D: Deserializer<'de>,
  {
    let micros = u64::deserialize(deserializer)?;
    Ok(Duration::from_micros(micros))
  }
}

/// Duration related statistics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct DurationStatistics {
  /// The maximum duration.
  #[serde(with = "as_micros")]
  pub max_time: Duration,
  /// The minimum duration.
  #[serde(with = "as_micros")]
  pub min_time: Duration,
  /// The average duration.
  #[serde(with = "as_micros")]
  pub average: Duration,
}

impl MessageKind {
  /// Converts a [MessageKind] into a [Packet].
  #[must_use]
  pub fn into_packet(self) -> Packet {
    self.into()
  }

  /// Converts a [MessageKind] into a [MessageTransport].
  #[must_use]
  pub fn into_transport(self) -> Packet {
    self.into()
  }

  /// Utility function to determine if [MessageKind] is a Signal.
  #[must_use]
  pub fn is_signal(&self) -> bool {
    let kind: Option<message_kind::Kind> = message_kind::Kind::from_i32(self.kind);
    matches!(kind, Some(message_kind::Kind::Signal))
  }
}

impl Output {
  /// Utility function to determine if [MessageKind] is a Signal.
  #[must_use]
  pub fn is_signal(&self) -> bool {
    let num = self.payload.as_ref().map_or(-1, |p| p.kind);
    let kind: Option<message_kind::Kind> = message_kind::Kind::from_i32(num);
    matches!(kind, Some(message_kind::Kind::Signal))
  }

  /// Convert the Output to JSON object value. This will not fail. If there is an error, the return value will be a serialized wrapper for a [MessageTransport::Error].
  #[must_use]
  pub fn into_json(self) -> serde_json::Value {
    let transport: TransportWrapper = self.into();
    transport.into_json()
  }

  /// Attempt to deserialize the payload into the destination type
  pub fn try_into<T: DeserializeOwned>(self) -> Result<T> {
    let transport: TransportWrapper = self.into();
    transport
      .try_into()
      .map_err(|e| RpcError::General(e.to_string()))
  }

  /// Convert the RPC output into a [TransportWrapper]
  pub fn into_transport_wrapper(self) -> TransportWrapper {
    self.into()
  }
}

impl From<Output> for TransportWrapper {
  fn from(v: Output) -> Self {
    Self {
      port: v.port,
      payload: v.payload.map_or(
        MessageTransport::Failure(Failure::Error("Could not decode RPC message".to_owned())),
        |p| p.into(),
      ),
    }
  }
}

impl From<MessageKind> for MessageTransport {
  fn from(v: MessageKind) -> Self {
    let packet: Packet = v.into();
    packet.into()
  }
}

impl From<MessageTransport> for MessageKind {
  fn from(v: MessageTransport) -> Self {
    let kind: i32 = match &v {
      MessageTransport::Success(v) => match v {
        vino_transport::Success::MessagePack(_) => message_kind::Kind::MessagePack,
        vino_transport::Success::Serialized(_) => message_kind::Kind::Json,
        vino_transport::Success::Json(_) => message_kind::Kind::Json,
      },
      MessageTransport::Failure(v) => match v {
        vino_transport::Failure::Invalid => message_kind::Kind::Invalid,
        vino_transport::Failure::Exception(_) => message_kind::Kind::Exception,
        vino_transport::Failure::Error(_) => message_kind::Kind::Error,
      },
      MessageTransport::Signal(_) => message_kind::Kind::Signal,
    }
    .into();
    let data = match v {
      MessageTransport::Success(v) => match v {
        vino_transport::Success::MessagePack(v) => Some(message_kind::Data::Messagepack(v)),
        vino_transport::Success::Serialized(val) => match vino_codec::json::serialize(&val) {
          Ok(json) => Some(message_kind::Data::Json(json)),
          Err(e) => Some(message_kind::Data::Message(e.to_string())),
        },
        vino_transport::Success::Json(json) => Some(message_kind::Data::Json(json)),
      },
      MessageTransport::Failure(v) => match v {
        vino_transport::Failure::Invalid => None,
        vino_transport::Failure::Exception(v) => Some(message_kind::Data::Message(v)),
        vino_transport::Failure::Error(v) => Some(message_kind::Data::Message(v)),
      },
      MessageTransport::Signal(signal) => match signal {
        vino_transport::MessageSignal::Done => Some(message_kind::Data::Signal(
          message_kind::OutputSignal::Done.into(),
        )),
        vino_transport::MessageSignal::OpenBracket => Some(message_kind::Data::Signal(
          message_kind::OutputSignal::OpenBracket.into(),
        )),
        vino_transport::MessageSignal::CloseBracket => Some(message_kind::Data::Signal(
          message_kind::OutputSignal::CloseBracket.into(),
        )),
      },
    };
    MessageKind { kind, data }
  }
}
