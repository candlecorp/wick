/// Conversion utilities for RPC data structures.
pub(crate) mod conversions;
use std::time::Duration;

pub use conversions::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use wasmflow_sdk::v1::codec::json;
use wasmflow_sdk::v1::packet::Packet;
use wasmflow_sdk::v1::transport::{Failure, MessageSignal, MessageTransport, Serialized, TransportWrapper};
pub use wasmflow_sdk::v1::types::*;

use crate::error::RpcError;
use crate::rpc::{self, packet as rpc_packet, Output, Packet as RpcPacket};
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

impl RpcPacket {
  /// Converts a [RpcPacket] into a [Packet].
  pub fn into_packet(self) -> Packet {
    self.into()
  }

  /// Converts a [RpcPacket] into a [MessageTransport].
  pub fn into_transport(self) -> Packet {
    self.into()
  }

  /// Utility function to determine if [RpcPacket] is a Signal.
  #[must_use]
  pub fn is_signal(&self) -> bool {
    assert!(self.data.is_some(), "Invalid RPC message contains no data");
    let data = self.data.as_ref().unwrap();
    matches!(data, rpc_packet::Data::Signal(_))
  }
}

impl Output {
  /// Utility function to determine if [RpcPacket] is a Signal.
  #[must_use]
  pub fn is_signal(&self) -> bool {
    assert!(
      self.payload.is_some(),
      "Invalid packet in Output stream, contains no data"
    );
    let packet = self.payload.as_ref().unwrap();

    matches!(packet.data, Some(rpc_packet::Data::Signal(_)))
  }

  /// Convert the Output to JSON object value. This will not fail. If there is an error, the return value will be a serialized wrapper for a [MessageTransport::Error].
  #[must_use]
  pub fn into_json(self) -> serde_json::Value {
    let transport: TransportWrapper = self.into();
    transport.as_json()
  }

  /// Attempt to deserialize the payload into the destination type
  pub fn try_into<T: DeserializeOwned>(self) -> Result<T> {
    let transport: TransportWrapper = self.into();
    transport.deserialize().map_err(|e| RpcError::General(e.to_string()))
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

impl From<RpcPacket> for MessageTransport {
  fn from(v: RpcPacket) -> Self {
    let packet: Packet = v.into();
    packet.into()
  }
}

impl From<MessageTransport> for RpcPacket {
  fn from(v: MessageTransport) -> Self {
    let data: rpc_packet::Data = match v {
      MessageTransport::Success(v) => match v {
        Serialized::MessagePack(v) => rpc_packet::Data::Success(rpc::Serialized {
          payload: Some(rpc::PayloadData {
            data: Some(rpc::payload_data::Data::Messagepack(v)),
          }),
        }),
        Serialized::Struct(v) => match json::serialize(&v) {
          Ok(json) => rpc_packet::Data::Success(rpc::Serialized {
            payload: Some(rpc::PayloadData {
              data: Some(rpc::payload_data::Data::Json(json)),
            }),
          }),
          Err(e) => rpc_packet::Data::Failure(rpc::Failure {
            r#type: rpc::failure::FailureKind::Error.into(),
            payload: e.to_string(),
          }),
        },
        Serialized::Json(v) => rpc_packet::Data::Success(rpc::Serialized {
          payload: Some(rpc::PayloadData {
            data: Some(rpc::payload_data::Data::Json(v)),
          }),
        }),
      },
      MessageTransport::Failure(v) => match v {
        Failure::Invalid => panic!("Invalid packet sent over GRPC"),
        Failure::Exception(v) => rpc_packet::Data::Failure(rpc::Failure {
          r#type: rpc::failure::FailureKind::Exception.into(),
          payload: v,
        }),
        Failure::Error(v) => rpc_packet::Data::Failure(rpc::Failure {
          r#type: rpc::failure::FailureKind::Error.into(),
          payload: v,
        }),
      },
      MessageTransport::Signal(signal) => match signal {
        MessageSignal::Done => rpc_packet::Data::Signal(rpc::Signal {
          r#type: rpc::signal::OutputSignal::Done.into(),
          payload: None,
        }),
        MessageSignal::OpenBracket => rpc_packet::Data::Signal(rpc::Signal {
          r#type: rpc::signal::OutputSignal::OpenBracket.into(),
          payload: None,
        }),
        MessageSignal::CloseBracket => rpc_packet::Data::Signal(rpc::Signal {
          r#type: rpc::signal::OutputSignal::CloseBracket.into(),
          payload: None,
        }),
      },
    };
    RpcPacket { data: Some(data) }
  }
}
