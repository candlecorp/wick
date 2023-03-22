/// Conversion utilities for RPC data structures.
pub(crate) mod conversions;
use std::time::Duration;

use serde::{Deserialize, Serialize};
pub use wick_interface_types::*;
use wick_packet::{Metadata, Packet, PacketError, PacketPayload, WickMetadata};

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
}

impl From<RpcPacket> for Packet {
  fn from(v: RpcPacket) -> Self {
    let (op, port, done) = v.metadata.map_or_else(
      || (0, "<component>".to_owned(), 0_u8),
      |m| (m.index, m.port, m.flags.try_into().unwrap()),
    );
    Self::new_raw(
      v.data
        .map_or(PacketPayload::fatal_error("Could not decode RPC message"), |p| p.into()),
      Metadata::new(op),
      WickMetadata::new(port, done),
    )
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
    }
  }
}
