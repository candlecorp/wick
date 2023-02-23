use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use wasmrs_frames::ex_err;

const DONE_FLAG: u8 = 0b1000_0000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct WickMetadata {
  done: bool,
  pub(crate) stream: String,
}

impl Default for WickMetadata {
  fn default() -> Self {
    Self {
      done: true,
      stream: "<component>".to_owned(),
    }
  }
}

impl WickMetadata {
  pub fn new(stream: impl AsRef<str>, done: bool) -> Self {
    Self {
      done,
      stream: stream.as_ref().to_owned(),
    }
  }
  pub fn new_done(stream: impl AsRef<str>) -> Self {
    Self {
      done: true,
      stream: stream.as_ref().to_owned(),
    }
  }

  #[must_use]
  pub fn stream(&self) -> &str {
    &self.stream
  }

  #[must_use]
  pub fn is_done(&self) -> bool {
    self.done
  }

  pub fn decode(mut bytes: Bytes) -> Result<Self, wasmrs_frames::Error> {
    let flags = bytes.get_u8();
    let done = flags & DONE_FLAG == DONE_FLAG;
    let name_len = bytes.get_u16();
    let name_bytes = bytes
      .get(0..(name_len as _))
      .ok_or_else(|| ex_err("Could not read stream name bytes"))?;
    let stream_name = String::from_utf8(name_bytes.to_vec()).map_err(|_| ex_err("Could not parse stream name"))?;
    Ok(WickMetadata::new(stream_name, done))
  }

  #[must_use]
  pub fn encode(self) -> Bytes {
    let mut bytes = BytesMut::new();
    let mut flags = 0_u8;
    if self.done {
      flags |= DONE_FLAG;
    }
    bytes.put_u8(flags);
    bytes.put_u16(self.stream.len() as _);
    bytes.put(self.stream.as_bytes());
    bytes.freeze()
  }
}

#[cfg(test)]
mod test {

  use anyhow::Result;

  use super::*;

  #[test]
  fn test_metadata_decode() -> Result<()> {
    let md = WickMetadata::new("left", true);
    println!("md: {:?}", md);
    let bytes = md.encode();
    println!("bytes: {:02x?}", bytes.to_vec());
    let meta = WickMetadata::decode(bytes)?;
    assert_eq!(meta.stream, "left");
    assert!(meta.done);
    Ok(())
  }
}
