use futures::StreamExt;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::debug;
use wasmrs::{BoxFlux, Metadata, Payload, PayloadError, RawPayload};
use wick_interface_types::TypeSignature;

use crate::metadata::DONE_FLAG;
use crate::{
  Base64Bytes,
  ComponentReference,
  Error,
  PacketStream,
  TypeWrapper,
  WickMetadata,
  CLOSE_BRACKET,
  OPEN_BRACKET,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
pub struct Packet {
  pub(crate) metadata: Metadata,
  pub(crate) extra: WickMetadata,
  pub payload: PacketPayload,
}

impl PartialEq for Packet {
  fn eq(&self, other: &Self) -> bool {
    if self.metadata.index != other.metadata.index || !self.metadata.extra.eq(&other.metadata.extra) {
      return false;
    }
    if self.extra.ne(&other.extra) {
      return false;
    }
    self.payload == other.payload
  }
}

impl Packet {
  pub fn new_raw(payload: PacketPayload, wasmrs: Metadata, metadata: WickMetadata) -> Self {
    Self {
      payload,
      metadata: wasmrs,
      extra: metadata,
    }
  }

  pub fn new_for_port(port: impl AsRef<str>, payload: PacketPayload, flags: u8) -> Self {
    let md = Metadata::new(0);
    let wmd = WickMetadata::new(port, flags);
    Self {
      payload,
      metadata: md,
      extra: wmd,
    }
  }

  pub fn component_error(err: impl AsRef<str>) -> Self {
    Self::new_for_port("<component>", PacketPayload::fatal_error(err), 0)
  }

  pub fn ok(port: impl AsRef<str>, payload: RawPayload) -> Self {
    Self::new_for_port(port, PacketPayload::Ok(payload.data.map(Into::into)), 0)
  }

  pub fn raw_err(port: impl AsRef<str>, payload: PacketError) -> Self {
    Self::new_for_port(port, PacketPayload::Err(payload), 0)
  }

  pub fn err(port: impl AsRef<str>, msg: impl AsRef<str>) -> Self {
    Self::new_for_port(port, PacketPayload::Err(PacketError::new(msg)), 0)
  }

  pub fn done(port: impl AsRef<str>) -> Self {
    Self::new_for_port(port, PacketPayload::Ok(None), DONE_FLAG)
  }

  pub fn open_bracket(port: impl AsRef<str>) -> Self {
    Self::new_for_port(port, PacketPayload::Ok(None), OPEN_BRACKET)
  }

  pub fn close_bracket(port: impl AsRef<str>) -> Self {
    Self::new_for_port(port, PacketPayload::Ok(None), CLOSE_BRACKET)
  }

  pub fn encode<T: Serialize>(port: impl AsRef<str>, data: T) -> Self {
    match wasmrs_codec::messagepack::serialize(&data) {
      Ok(bytes) => Self::new_for_port(port, PacketPayload::Ok(Some(bytes.into())), 0),
      Err(err) => Self::new_for_port(port, PacketPayload::Err(PacketError::new(err.to_string())), 0),
    }
  }

  pub fn flags(&self) -> u8 {
    self.extra.flags
  }

  pub fn index(&self) -> u32 {
    self.metadata.index
  }

  /// Try to deserialize a [Packet] into the target type
  pub fn deserialize<T: DeserializeOwned>(self) -> Result<T, Error> {
    self.payload.deserialize()
  }

  /// Try to deserialize a [Packet] into the target type
  pub fn deserialize_into(self, ty: TypeSignature) -> Result<TypeWrapper, Error> {
    self.payload.deserialize_into(ty)
  }

  pub fn deserialize_generic(self) -> Result<serde_json::Value, Error> {
    self.payload.deserialize()
  }

  pub fn set_port(mut self, port: impl AsRef<str>) -> Self {
    self.extra.port = port.as_ref().to_owned();
    self
  }

  pub fn port(&self) -> &str {
    &self.extra.port
  }

  pub fn payload(&self) -> &PacketPayload {
    &self.payload
  }

  pub fn is_done(&self) -> bool {
    self.extra.is_done()
  }

  pub fn is_open_bracket(&self) -> bool {
    self.extra.is_open_bracket()
  }

  pub fn is_close_bracket(&self) -> bool {
    self.extra.is_close_bracket()
  }

  pub fn to_json(&self) -> serde_json::Value {
    if self.flags() > 0 {
      serde_json::json!({
        "flags": self.flags(),
        "port": self.port(),
        "payload": self.payload.to_json(),
      })
    } else {
      serde_json::json!({
        "port": self.port(),
        "payload": self.payload.to_json(),
      })
    }
  }

  pub fn from_kv_json(values: &[String]) -> Result<Vec<Packet>, Error> {
    let mut packets = Vec::new();

    for input in values {
      match input.split_once('=') {
        Some((port, value)) => {
          debug!(port, value, "cli:args:port-data");
          let val: serde_json::Value =
            serde_json::from_str(value).map_err(|e| crate::Error::Decode(value.as_bytes().to_vec(), e.to_string()))?;
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
  Ok(Option<Base64Bytes>),
  Err(PacketError),
}

impl PacketPayload {
  pub fn fatal_error(err: impl AsRef<str>) -> Self {
    Self::Err(PacketError::new(err))
  }

  pub fn serialize<T: Serialize>(data: T) -> Self {
    match wasmrs_codec::messagepack::serialize(&data) {
      Ok(bytes) => Self::Ok(Some(bytes.into())),
      Err(err) => Self::Err(PacketError::new(err.to_string())),
    }
  }

  /// Try to deserialize a [Packet] into the target type
  pub fn deserialize<T: DeserializeOwned>(self) -> Result<T, Error> {
    match self {
      PacketPayload::Ok(Some(bytes)) => match wasmrs_codec::messagepack::deserialize(&bytes) {
        Ok(data) => Ok(data),
        Err(err) => Err(crate::Error::Decode(bytes.into(), err.to_string())),
      },
      PacketPayload::Ok(None) => Err(crate::Error::NoData),
      PacketPayload::Err(err) => Err(crate::Error::PayloadError(err)),
    }
  }

  /// Try to deserialize a [Packet] into the target type
  pub fn deserialize_into(self, sig: TypeSignature) -> Result<TypeWrapper, Error> {
    let val = match sig {
      TypeSignature::I8 => TypeWrapper::new(sig, self.deserialize::<i8>()?.into()),
      TypeSignature::I16 => TypeWrapper::new(sig, self.deserialize::<i16>()?.into()),
      TypeSignature::I32 => TypeWrapper::new(sig, self.deserialize::<i32>()?.into()),
      TypeSignature::I64 => TypeWrapper::new(sig, self.deserialize::<i64>()?.into()),
      TypeSignature::U8 => TypeWrapper::new(sig, self.deserialize::<u8>()?.into()),
      TypeSignature::U16 => TypeWrapper::new(sig, self.deserialize::<u16>()?.into()),
      TypeSignature::U32 => TypeWrapper::new(sig, self.deserialize::<u32>()?.into()),
      TypeSignature::U64 => TypeWrapper::new(sig, self.deserialize::<u64>()?.into()),
      TypeSignature::F32 => TypeWrapper::new(sig, self.deserialize::<f32>()?.into()),
      TypeSignature::F64 => TypeWrapper::new(sig, self.deserialize::<f64>()?.into()),
      TypeSignature::Bool => TypeWrapper::new(sig, self.deserialize::<bool>()?.into()),
      TypeSignature::String => TypeWrapper::new(sig, self.deserialize::<String>()?.into()),
      TypeSignature::Datetime => TypeWrapper::new(sig, self.deserialize::<String>()?.into()),
      TypeSignature::Bytes => TypeWrapper::new(sig, self.deserialize::<Vec<u8>>()?.into()),
      TypeSignature::Custom(_) => TypeWrapper::new(sig, self.deserialize::<serde_json::Value>()?),
      TypeSignature::Ref { .. } => unimplemented!(),
      TypeSignature::Stream { .. } => unimplemented!(),
      TypeSignature::List { .. } => TypeWrapper::new(sig, self.deserialize::<Vec<serde_json::Value>>()?.into()),
      TypeSignature::Optional { .. } => TypeWrapper::new(sig, self.deserialize::<Option<serde_json::Value>>()?.into()),
      TypeSignature::Map { .. } => TypeWrapper::new(
        sig,
        serde_json::Value::Object(self.deserialize::<serde_json::Map<String, serde_json::Value>>()?),
      ),
      TypeSignature::Link { .. } => TypeWrapper::new(
        sig,
        serde_json::Value::String(self.deserialize::<ComponentReference>()?.to_string()),
      ),
      TypeSignature::Object => TypeWrapper::new(sig, self.deserialize::<serde_json::Value>()?),
      TypeSignature::AnonymousStruct(_) => unimplemented!(),
    };
    Ok(val)
  }

  pub fn bytes(&self) -> Option<&Base64Bytes> {
    match self {
      Self::Ok(b) => b.as_ref(),
      _ => None,
    }
  }

  pub fn into_bytes(self) -> Option<Base64Bytes> {
    match self {
      Self::Ok(b) => b,
      _ => None,
    }
  }

  pub fn to_json(&self) -> serde_json::Value {
    match self {
      Self::Ok(Some(b)) => match wasmrs_codec::messagepack::deserialize::<serde_json::Value>(b) {
        Ok(data) => serde_json::json!({ "value": data }),
        Err(err) => serde_json::json! ({"error" : crate::Error::Jsonify(err.to_string()).to_string()}),
      },
      Self::Ok(None) => serde_json::Value::Null,
      Self::Err(err) => serde_json::json! ({"error" : crate::Error::PayloadError(err.clone()).to_string()}),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[must_use]
pub fn into_wasmrs(index: u32, stream: PacketStream) -> BoxFlux<RawPayload, PayloadError> {
  let s = StreamExt::map(stream, move |p| {
    p.map(|p| {
      let md = wasmrs::Metadata::new_extra(index, p.extra.encode()).encode();
      match p.payload {
        PacketPayload::Ok(b) => Ok(wasmrs::RawPayload::new_data(Some(md), b.map(Into::into))),
        PacketPayload::Err(e) => Err(wasmrs::PayloadError::application_error(e.msg(), Some(md))),
      }
    })
    .unwrap_or(Err(PayloadError::application_error("failed", None)))
  });
  Box::pin(s)
}

pub fn from_raw_wasmrs(stream: BoxFlux<RawPayload, PayloadError>) -> PacketStream {
  let s = StreamExt::map(stream, move |p| {
    let p = p.map_or_else(
      |e| {
        let md = wasmrs::Metadata::decode(&mut e.metadata.unwrap());

        let wmd = md.map_or_else(
          |_e| WickMetadata::default(),
          |m| WickMetadata::decode(m.extra.unwrap()).unwrap(),
        );
        Packet::raw_err(wmd.port, PacketError::new(e.msg))
      },
      |p| {
        let md = wasmrs::Metadata::decode(&mut p.metadata.unwrap());
        let wmd = md.map_or_else(
          |_e| WickMetadata::default(),
          |m| WickMetadata::decode(m.extra.unwrap()).unwrap(),
        );
        // Potential danger zone: this converts empty payload to None which *should* be the
        // same thing. Calling this out as a potential source for weird bugs if they pop up.
        let data = p.data.and_then(|b| (!b.is_empty()).then_some(b));
        Packet::new_for_port(wmd.port(), PacketPayload::Ok(data.map(Into::into)), wmd.flags())
      },
    );
    Ok(p)
  });

  PacketStream::new(Box::new(s))
}

pub fn from_wasmrs(stream: BoxFlux<Payload, PayloadError>) -> PacketStream {
  let s = StreamExt::map(stream, move |p| {
    let p = p.map_or_else(
      |e| {
        let md = wasmrs::Metadata::decode(&mut e.metadata.unwrap());

        let wmd = md.map_or_else(
          |_e| WickMetadata::default(),
          |m| WickMetadata::decode(m.extra.unwrap()).unwrap(),
        );
        Packet::raw_err(wmd.port, PacketError::new(e.msg))
      },
      |p| {
        let md = p.metadata;
        let wmd = WickMetadata::decode(md.extra.unwrap()).unwrap();
        // Potential danger zone: this converts empty payload to None which *should* be the
        // same thing. Calling this out as a potential source for weird bugs if they pop up.
        let data = p.data;
        Packet::new_for_port(wmd.port(), PacketPayload::Ok(Some(data.into())), wmd.flags())
      },
    );
    Ok(p)
  });
  PacketStream::new(Box::new(s))
}

impl From<Payload> for Packet {
  fn from(mut value: Payload) -> Self {
    let ex = value.metadata.extra.take();
    Self {
      extra: WickMetadata::decode(ex.unwrap()).unwrap(),
      metadata: value.metadata,
      payload: PacketPayload::Ok(Some(value.data.into())),
    }
  }
}

impl From<Packet> for Result<RawPayload, PayloadError> {
  fn from(value: Packet) -> Self {
    let mut md = value.metadata;
    md.extra = Some(value.extra.encode());
    match value.payload {
      PacketPayload::Ok(b) => Ok(RawPayload::new_data(Some(md.encode()), b.map(Into::into))),
      PacketPayload::Err(e) => Err(PayloadError::application_error(e.msg(), Some(md.encode()))),
    }
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use serde_json::Value;

  use crate::{Base64Bytes, Packet};

  #[test]
  fn test_basic() -> Result<()> {
    let packet = Packet::encode("test", 10);
    let res: i32 = packet.deserialize()?;
    assert_eq!(res, 10);
    Ok(())
  }

  #[rstest::rstest]
  #[case(u64::MIN, Value::Number(serde_json::Number::from(u64::MIN)))]
  #[case(u64::MAX, Value::Number(serde_json::Number::from(u64::MAX)))]
  #[case(&[1,2,3,4,5,6], vec![1,2,3,4,5,6].into())]
  #[case("test", Value::String("test".to_owned()))]
  #[case(Base64Bytes::new(b"test".as_slice()), Value::String("dGVzdA==".to_owned()))]
  fn test_encode_to_generic<T>(#[case] value: T, #[case] expected: Value) -> Result<()>
  where
    T: serde::Serialize + std::fmt::Debug,
  {
    let packet = Packet::encode("test", value);
    println!("{:?}", packet);
    let res = packet.deserialize_generic()?;
    assert_eq!(res, expected);
    Ok(())
  }

  #[rstest::rstest]
  #[case("dGVzdA==", b"test")]
  fn test_from_b64(#[case] value: &str, #[case] expected: &[u8]) -> Result<()> {
    let packet = Packet::encode("test", value);
    println!("{:?}", packet);
    let res = packet.deserialize_generic()?;
    let bytes: Base64Bytes = serde_json::from_value(res).unwrap();
    assert_eq!(bytes, expected);
    Ok(())
  }
}
