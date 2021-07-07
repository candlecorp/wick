use std::io::Cursor;

use rmp_serde::{
  Deserializer,
  Serializer,
};
use serde::{
  Deserialize,
  Serialize,
};

use crate::error::CodecError;
use crate::Result;

#[doc(hidden)]
pub fn rmp_serialize<T>(item: T) -> std::result::Result<Vec<u8>, rmp_serde::encode::Error>
where
  T: Serialize,
{
  let mut buf = Vec::new();
  match item.serialize(&mut Serializer::new(&mut buf).with_string_variants()) {
    Ok(_) => Ok(buf),
    Err(e) => Err(e),
  }
}

/// The standard function for serializing codec structs into a format that can be
/// used for message exchange between actor and host. Use of any other function to
/// serialize could result in breaking incompatibilities.
pub fn serialize<T>(item: T) -> Result<Vec<u8>>
where
  T: Serialize,
{
  rmp_serialize(item).map_err(CodecError::SerializationError)
}

/// The standard function for de-serializing codec structs from a format suitable
/// for message exchange between actor and host. Use of any other function to
/// deserialize could result in breaking incompatibilities.
pub fn deserialize<'de, T: Deserialize<'de>>(buf: &[u8]) -> Result<T> {
  let mut de = Deserializer::new(Cursor::new(buf));
  match Deserialize::deserialize(&mut de) {
    Ok(t) => Ok(t),
    Err(e) => Err(CodecError::DeserializationError(e)),
  }
}
