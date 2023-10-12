use serde::de::DeserializeOwned;
use wasmrs_runtime::ConditionallySend;

use crate::{Error, Packet, PacketError, PacketExt, PacketPayload};

#[derive(Debug)]
pub struct VPacket<T>
where
  T: ConditionallySend,
{
  pub(crate) packet: Option<Result<Packet, anyhow::Error>>,
  pub(crate) value: Option<T>,
}

impl<T> VPacket<T>
where
  T: ConditionallySend,
{
  pub fn new(packet: Packet) -> Self
  where
    T: DeserializeOwned,
  {
    Self {
      value: Default::default(),
      packet: Some(Ok(packet)),
    }
  }

  pub fn from_result(packet: Result<Packet, anyhow::Error>) -> Self
  where
    T: DeserializeOwned + ConditionallySend,
  {
    Self {
      value: Default::default(),
      packet: Some(packet),
    }
  }

  pub const fn from_value(value: T) -> Self {
    VPacket {
      packet: None,
      value: Some(value),
    }
  }

  pub fn decode(self) -> Result<T, Error>
  where
    T: DeserializeOwned + ConditionallySend,
  {
    match self.value {
      Some(value) => Ok(value),
      None => match self.packet {
        Some(Ok(packet)) => packet.decode::<T>(),
        Some(Err(e)) => Err(Error::Component(e.to_string())),
        None => unreachable!("tried to decode VPacket that had no packet or value"),
      },
    }
  }
}

impl<T> From<Result<Packet, anyhow::Error>> for VPacket<T>
where
  T: DeserializeOwned + ConditionallySend,
{
  fn from(packet: Result<Packet, anyhow::Error>) -> Self {
    Self::from_result(packet)
  }
}

impl<T> From<VPacket<T>> for PacketPayload
where
  T: ConditionallySend + serde::Serialize,
{
  fn from(packet: VPacket<T>) -> Self {
    match packet.packet {
      Some(Ok(p)) => p.payload,
      Some(Err(e)) => PacketPayload::Err(PacketError::new(e.to_string())),
      None => packet.value.map_or_else(
        || unreachable!("tried to convert VPacket that had no packet or value"),
        |v| PacketPayload::encode(v),
      ),
    }
  }
}

impl<T> PacketExt for VPacket<T>
where
  T: ConditionallySend,
{
  fn has_data(&self) -> bool {
    match self.value {
      Some(_) => true,
      None => self
        .packet
        .as_ref()
        .map_or(false, |p| p.as_ref().map_or(false, |p| p.has_data())),
    }
  }

  fn port(&self) -> &str {
    self.packet.as_ref().map_or_else(
      || panic!("tried to query port for VPacket created manually"),
      |p| p.as_ref().map_or(Packet::FATAL_ERROR, |p| p.port()),
    )
  }

  fn flags(&self) -> u8 {
    match self.value {
      Some(_) => 0,
      None => self.packet.as_ref().map_or(0, |p| p.as_ref().map_or(0, |p| p.flags())),
    }
  }
}
