/// Conversion utilities for RPC data structures.
pub(crate) mod conversions;
use std::time::Duration;

use serde::{Deserialize, Serialize};
pub use wasmflow_interface::*;
use wasmflow_packet_stream::{Metadata, Packet, PacketError, PacketPayload, WickMetadata};

use crate::rpc::{self, packet as rpc_packet, Packet as RpcPacket};

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
  pub average_time: Duration,
  /// The total duration.
  #[serde(with = "as_micros")]
  pub total_time: Duration,
}

impl RpcPacket {
  /// Converts a [RpcPacket] into a [Packet].
  pub fn into_packet(self) -> Packet {
    self.into()
  }

  /// Utility function to determine if [RpcPacket] is a Signal.
  #[must_use]
  pub fn is_signal(&self) -> bool {
    assert!(self.data.is_some(), "Invalid RPC message contains no data");
    let data = self.data.as_ref().unwrap();
    matches!(data, rpc_packet::Data::Done(_))
  }
}

// impl Output {
//   /// Utility function to determine if [RpcPacket] is a Signal.
//   #[must_use]
//   pub fn is_signal(&self) -> bool {
//     assert!(
//       self.payload.is_some(),
//       "Invalid packet in Output stream, contains no data"
//     );
//     let packet = self.payload.as_ref().unwrap();

//     matches!(packet.data, Some(rpc_packet::Data::Signal(_)))
//   }

//   /// Convert the Output to JSON object value. This will not fail. If there is an error, the return value will be a serialized wrapper for a [MessageTransport::Error].
//   #[must_use]
//   pub fn into_json(self) -> serde_json::Value {
//     let transport: TransportWrapper = self.into();
//     transport.as_json()
//   }

//   /// Attempt to deserialize the payload into the destination type
//   pub fn try_into<T: DeserializeOwned>(self) -> Result<T> {
//     let transport: TransportWrapper = self.into();
//     transport.deserialize().map_err(|e| RpcError::General(e.to_string()))
//   }

//   /// Convert the RPC output into a [TransportWrapper]
//   pub fn into_transport_wrapper(self) -> TransportWrapper {
//     self.into()
//   }
// }

impl From<RpcPacket> for Packet {
  fn from(v: RpcPacket) -> Self {
    let (op, port, done) = v
      .metadata
      .map_or_else(|| (0, "<component>".to_owned(), true), |m| (m.index, m.port, m.done));
    Self {
      // todo figure out operation indexes still
      metadata: Metadata::new(op),
      extra: WickMetadata::new(port, done),
      payload: v
        .data
        .map_or(PacketPayload::fatal_error("Could not decode RPC message"), |p| p.into()),
    }
  }
}

impl From<rpc_packet::Data> for PacketPayload {
  fn from(v: rpc_packet::Data) -> Self {
    match v {
      rpc_packet::Data::Ok(v) => PacketPayload::Ok(match v.data {
        Some(rpc::ok::Data::Messagepack(v)) => v.into(),
        Some(rpc::ok::Data::Json(_v)) => todo!(),
        None => unreachable!(),
      }),

      rpc_packet::Data::Err(v) => PacketPayload::Err(PacketError::new(v.message)),
      rpc_packet::Data::Done(_) => PacketPayload::Done,
    }
  }
}

impl From<PacketPayload> for rpc_packet::Data {
  fn from(v: PacketPayload) -> Self {
    match v {
      PacketPayload::Ok(v) => rpc_packet::Data::Ok(rpc::Ok {
        data: Some(rpc::ok::Data::Messagepack(v.to_vec())),
      }),
      PacketPayload::Err(e) => rpc_packet::Data::Err(rpc::Err {
        message: e.msg().to_owned(),
        code: 513,
      }),
      PacketPayload::Done => rpc_packet::Data::Done(true),
    }
  }
}
