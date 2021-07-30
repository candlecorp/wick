use serde::{
  Deserialize,
  Serialize,
};
use serde_value::to_value;

use crate::error::CodecError;
use crate::Result;

#[doc(hidden)]
pub fn raw_serialize<T>(
  item: &T,
) -> std::result::Result<serde_value::Value, serde_value::SerializerError>
where
  T: Serialize,
{
  to_value(item)
}

/// The standard function for serializing codec structs into a format that can be
/// used for message exchange between actor and host. Use of any other function to
/// serialize could result in breaking incompatibilities.
pub fn serialize<T>(item: &T) -> Result<serde_value::Value>
where
  T: Serialize,
{
  raw_serialize(item).map_err(CodecError::SerializationError)
}

#[doc(hidden)]
pub fn raw_deserialize<'de, T: Deserialize<'de>>(
  value: serde_value::Value,
) -> std::result::Result<T, serde_value::DeserializerError> {
  value.deserialize_into()
}

/// The standard function for de-serializing codec structs from a format suitable
/// for message exchange between actor and host. Use of any other function to
/// deserialize could result in breaking incompatibilities.
pub fn deserialize<'de, T: Deserialize<'de>>(value: serde_value::Value) -> Result<T> {
  raw_deserialize(value).map_err(CodecError::DeserializationError)
}
