use bytes::Bytes;
use futures::StreamExt;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::debug;
use wasmrs::{Metadata, ParsedPayload, Payload, PayloadError};
use wasmrs_rx::FluxReceiver;

use crate::{Error, PacketStream, WickMetadata};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
pub struct Packet {
  pub metadata: Metadata,
  pub extra: WickMetadata,
  pub payload: PacketPayload,
}

impl PartialEq for Packet {
  fn eq(&self, other: &Self) -> bool {
    if self.metadata.index != other.metadata.index || !self.metadata.extra.eq(&other.metadata.extra) {
      return false;
    }
    self.payload == other.payload
  }
}

impl Packet {
  // pub fn new(payload: PacketPayload, metadata: Metadata) -> Self {
  //   Self { payload, metadata }
  // }

  pub fn new_for_port(port: impl AsRef<str>, payload: PacketPayload) -> Self {
    let md = Metadata::new(0);
    let wmd = WickMetadata::new(port, matches!(payload, PacketPayload::Done));
    Self {
      payload,
      metadata: md,
      extra: wmd,
    }
  }

  pub fn component_error(err: impl AsRef<str>) -> Self {
    Self::new_for_port("<component>", PacketPayload::fatal_error(err))
  }

  pub fn ok(port: impl AsRef<str>, payload: Payload) -> Self {
    Self::new_for_port(port, PacketPayload::Ok(payload.data.unwrap()))
  }

  pub fn raw_err(port: impl AsRef<str>, payload: PacketError) -> Self {
    Self::new_for_port(port, PacketPayload::Err(payload))
  }

  pub fn err(port: impl AsRef<str>, msg: impl AsRef<str>) -> Self {
    Self::new_for_port(port, PacketPayload::Err(PacketError::new(msg)))
  }

  pub fn done(port: impl AsRef<str>) -> Self {
    Self::new_for_port(port, PacketPayload::Done)
  }

  pub fn encode<T: Serialize>(port: impl AsRef<str>, data: T) -> Self {
    match wasmrs_codec::messagepack::serialize(&data) {
      Ok(bytes) => Self::new_for_port(port, PacketPayload::Ok(bytes.into())),
      Err(err) => Self::new_for_port(port, PacketPayload::Err(PacketError::new(err.to_string()))),
    }
  }

  /// Try to deserialize a [Packet] into the target type
  pub fn deserialize<T: DeserializeOwned>(self) -> Result<T, Error> {
    match self.payload {
      PacketPayload::Ok(bytes) => match wasmrs_codec::messagepack::deserialize(&bytes) {
        Ok(data) => Ok(data),
        Err(err) => Err(crate::Error::Codec(err.to_string())),
      },
      PacketPayload::Err(err) => Err(crate::Error::PayloadError(err)),
      PacketPayload::Done => Err(crate::Error::UnexpectedDone),
    }
  }
  pub fn set_port(mut self, port: impl AsRef<str>) -> Self {
    self.extra.stream = port.as_ref().to_owned();
    self
  }

  pub fn port_name(&self) -> &str {
    &self.extra.stream
  }

  pub fn payload(&self) -> &PacketPayload {
    &self.payload
  }

  pub fn is_done(&self) -> bool {
    matches!(self.payload, PacketPayload::Done)
  }

  pub fn from_kv_json(values: &[String]) -> Result<Vec<Packet>, Error> {
    let mut packets = Vec::new();

    for input in values {
      match input.split_once('=') {
        Some((port, value)) => {
          debug!(port, value, "cli:args:port-data");
          let val: serde_json::Value = serde_json::from_str(value).map_err(|e| crate::Error::Codec(e.to_string()))?;
          packets.push(Packet::encode(port, val));
        }
        None => return Err(Error::General(format!("Invalid port=value pair: '{}'", input))),
      }
    }
    Ok(packets)
  }
}

impl PartialEq for PacketPayload {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Ok(l0), Self::Ok(r0)) => l0 == r0,
      (Self::Err(l0), Self::Err(r0)) => l0.msg == r0.msg,
      _ => core::mem::discriminant(self) == core::mem::discriminant(other),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PacketPayload {
  Ok(Bytes),
  Err(PacketError),
  Done,
}

impl PacketPayload {
  pub fn fatal_error(err: impl AsRef<str>) -> Self {
    Self::Err(PacketError::new(err))
  }

  pub fn serialize<T: Serialize>(data: T) -> Self {
    match wasmrs_codec::messagepack::serialize(&data) {
      Ok(bytes) => Self::Ok(bytes.into()),
      Err(err) => Self::Err(PacketError::new(err.to_string())),
    }
  }

  /// Try to deserialize a [Packet] into the target type
  pub fn deserialize<T: DeserializeOwned>(self) -> Result<T, Error> {
    match self {
      Self::Ok(bytes) => match wasmrs_codec::messagepack::deserialize(&bytes) {
        Ok(data) => Ok(data),
        Err(err) => Err(crate::Error::Codec(err.to_string())),
      },
      Self::Err(err) => Err(crate::Error::PayloadError(err)),
      Self::Done => Err(crate::Error::UnexpectedDone),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketError {
  msg: String,
}

impl PacketError {
  pub fn new(msg: impl AsRef<str>) -> Self {
    Self {
      msg: msg.as_ref().to_owned(),
    }
  }

  #[must_use]
  pub fn msg(&self) -> &str {
    &self.msg
  }
}

pub fn into_wasmrs(index: u32, stream: PacketStream) -> Box<dyn wasmrs::Flux<Payload, PayloadError>> {
  let s = StreamExt::map(stream, move |p| {
    p.map(|p| {
      let md = wasmrs::Metadata::new_extra(index, p.extra.encode()).encode();
      match p.payload {
        PacketPayload::Ok(b) => Ok(wasmrs::Payload::new_data(Some(md), Some(b))),
        PacketPayload::Err(e) => Err(wasmrs::PayloadError::application_error(e.msg(), None)),
        PacketPayload::Done => Ok(wasmrs::Payload::new_data(Some(md), None)),
      }
    })
    .unwrap_or(Err(PayloadError::application_error("failed", None)))
  });
  Box::new(s)
}

pub fn from_wasmrs(stream: FluxReceiver<Payload, PayloadError>) -> PacketStream {
  let s = StreamExt::map(stream, move |p| {
    let p = p.map_or_else(
      |e| {
        let md = wasmrs::Metadata::decode(&mut e.metadata.unwrap());

        let wmd = md.map_or_else(
          |_e| WickMetadata::default(),
          |m| WickMetadata::decode(m.extra.unwrap()).unwrap(),
        );
        Packet::raw_err(wmd.stream, PacketError::new(e.msg))
      },
      |p| {
        let md = wasmrs::Metadata::decode(&mut p.metadata.unwrap());
        let wmd = md.map_or_else(
          |_e| WickMetadata::default(),
          |m| WickMetadata::decode(m.extra.unwrap()).unwrap(),
        );
        if wmd.is_done() {
          Packet::done(wmd.stream)
        } else {
          Packet::new_for_port(wmd.stream, PacketPayload::Ok(p.data.unwrap()))
        }
      },
    );
    Ok(p)
  });
  PacketStream::new(Box::new(s))
}

impl From<ParsedPayload> for Packet {
  fn from(mut value: ParsedPayload) -> Self {
    let ex = value.metadata.extra.take();
    Self {
      extra: WickMetadata::decode(ex.unwrap()).unwrap(),
      metadata: value.metadata,
      payload: PacketPayload::Ok(value.data),
    }
  }
}

impl From<Packet> for Result<Payload, PayloadError> {
  fn from(value: Packet) -> Self {
    let mut md = value.metadata;
    md.extra = Some(value.extra.encode());
    match value.payload {
      PacketPayload::Ok(b) => Ok(Payload::new(md.encode(), b)),
      PacketPayload::Err(e) => Err(PayloadError::application_error(e.msg(), None)),
      PacketPayload::Done => Ok(Payload::new_data(Some(md.encode()), None)),
    }
  }
}
