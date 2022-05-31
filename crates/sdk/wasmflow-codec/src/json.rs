use serde::{Deserialize, Serialize};

use crate::error::CodecError;
use crate::Result;

#[doc(hidden)]
pub fn to_value<T>(item: &T) -> std::result::Result<serde_json::Value, serde_json::Error>
where
  T: Serialize,
{
  serde_json::to_value(item)
}

#[doc(hidden)]
pub fn json_serialize<T>(item: &T) -> std::result::Result<String, serde_json::Error>
where
  T: Serialize,
{
  serde_json::to_string(item)
}

/// The standard function for serializing codec structs into a format that can be.
/// used for message exchange between actor and host. Use of any other function to.
/// serialize could result in breaking incompatibilities.
pub fn serialize<T>(item: &T) -> Result<String>
where
  T: Serialize,
{
  json_serialize(item).map_err(CodecError::JsonSerializationError)
}

#[doc(hidden)]
pub fn json_deserialize<'de, T: Deserialize<'de>>(
  json: &'de str,
) -> std::result::Result<T, serde_json::Error> {
  serde_json::from_str(json)
}

/// The standard function for de-serializing codec structs from a format suitable.
/// for message exchange between actor and host. Use of any other function to.
/// deserialize could result in breaking incompatibilities.
pub fn deserialize<'de, T: Deserialize<'de>>(json: &'de str) -> Result<T> {
  json_deserialize(json).map_err(CodecError::JsonDeserializationError)
}
