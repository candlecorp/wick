use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use wasmrs_frames::ex_err;

use crate::Base64Bytes;

pub const DONE_FLAG: u8 = /******/ 0b1000_0000;
pub const OPEN_BRACKET: u8 = /***/ 0b0100_0000;
pub const CLOSE_BRACKET: u8 = /**/ 0b0010_0000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[allow(clippy::exhaustive_enums)]
pub enum Flags {
  Done,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct WickMetadata {
  pub(crate) flags: u8,
  pub(crate) port: String,
  pub(crate) context: Option<Base64Bytes>,
}

impl Default for WickMetadata {
  fn default() -> Self {
    Self {
      flags: 0,
      port: crate::Packet::FATAL_ERROR.to_owned(),
      context: None,
    }
  }
}

impl WickMetadata {
  pub fn new<T: Into<String>>(port: T, flags: u8) -> Self {
    Self {
      flags,
      port: port.into(),
      context: None,
    }
  }

  #[must_use]
  pub const fn flags(&self) -> u8 {
    self.flags
  }

  pub fn set_context(&mut self, config: Base64Bytes) {
    self.context = Some(config);
  }

  #[must_use]
  pub fn port(&self) -> &str {
    &self.port
  }

  #[must_use]
  pub const fn is_done(&self) -> bool {
    self.flags & DONE_FLAG == DONE_FLAG
  }

  #[must_use]
  pub const fn is_open_bracket(&self) -> bool {
    self.flags & OPEN_BRACKET == OPEN_BRACKET
  }

  #[must_use]
  pub const fn is_close_bracket(&self) -> bool {
    self.flags & CLOSE_BRACKET == CLOSE_BRACKET
  }

  pub fn decode(mut bytes: Bytes) -> Result<Self, wasmrs_frames::Error> {
    let flags = bytes.get_u8();
    let name_len = bytes.get_u16();
    let name_bytes = bytes
      .get(0..(name_len as _))
      .ok_or_else(|| ex_err("Could not read port name bytes"))?;
    let port_name = String::from_utf8(name_bytes.to_vec()).map_err(|_| ex_err("Could not parse port name"))?;
    bytes.advance(name_len.into());
    let config_len = bytes.get_u16();
    let config_bytes = if config_len > 0 {
      bytes.get(0..(config_len as _)).map(|v| v.to_vec())
    } else {
      None
    };
    Ok(WickMetadata {
      flags,
      port: port_name,
      context: config_bytes.map(Into::into),
    })
  }

  #[must_use]
  pub fn encode(self) -> Bytes {
    let mut bytes = BytesMut::new();
    bytes.put_u8(self.flags);
    bytes.put_u16(self.port.len() as _);
    bytes.put(self.port.as_bytes());
    let config = self.context.unwrap_or_default();
    bytes.put_u16(config.len() as _);
    bytes.put(config);
    bytes.freeze()
  }
}

#[cfg(test)]
mod test {

  use anyhow::Result;

  use super::*;

  #[test]
  fn test_metadata_decode() -> Result<()> {
    let mut md = WickMetadata::new("left", DONE_FLAG | CLOSE_BRACKET);
    md.set_context(b"hello".to_vec().into());
    println!("md: {:?}", md);
    let bytes = md.encode();
    println!("bytes: {:02x?}", bytes.to_vec());
    let meta = WickMetadata::decode(bytes)?;
    assert_eq!(meta.port, "left");
    assert!(meta.is_done());
    assert!(meta.is_close_bracket());
    assert_eq!(meta.context.unwrap(), b"hello".to_vec());
    Ok(())
  }
}
