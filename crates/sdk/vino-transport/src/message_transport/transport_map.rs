use std::collections::HashMap;
use std::fmt::Display;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use wasmflow_codec::{json, messagepack, raw};

use crate::error::TransportError;
use crate::{Error, MessageTransport, Serialized, TransportWrapper};
pub(crate) type Result<T> = std::result::Result<T, TransportError>;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[must_use]
/// A wrapper for a map of [String]s to [MessageTransport]
pub struct TransportMap(HashMap<String, MessageTransport>, Option<HashMap<String, String>>);

impl TransportMap {
  wasmflow_macros::kv_impl! {MessageTransport, pub}

  /// Add a configuration payload to the [TransportMap].
  #[must_use]
  pub fn into_v1_map(self) -> wasmflow_packet::v1::PacketMap {
    let mut map = wasmflow_packet::v1::PacketMap::default();
    for (k, v) in self.0 {
      map.insert(k, v.into());
    }
    map
  }

  /// Deserialize a CLI output JSON Object into a [TransportMap].
  pub fn from_json_output(json: &str) -> Result<Self> {
    if json.trim() == "" {
      Ok(TransportMap::default())
    } else {
      let json: HashMap<String, super::transport_json::TransportJson> = json::deserialize(json)?;
      Ok(json.into_iter().map(|(name, val)| (name, val.into())).collect())
    }
  }

  /// Deserialize a JSON Object into a [TransportMap]
  pub fn from_json_str(json: &str) -> Result<Self> {
    if json.trim() == "" {
      Ok(TransportMap::default())
    } else {
      let map = serde_json::from_str::<HashMap<String, serde_json::Value>>(json)?;
      Ok(
        map
          .into_iter()
          .map(|(name, val)| (name, MessageTransport::json(&val.to_string())))
          .collect(),
      )
    }
  }

  /// Turn a list of "field=value" strings into a [TransportMap] of [Serialized::Json] items.
  pub fn from_kv_json(values: &[String]) -> Result<Self> {
    let mut payload = TransportMap::default();
    for input in values {
      match input.split_once('=') {
        Some((name, value)) => {
          debug!("PORT:'{}', VALUE:'{}'", name, value);
          payload.insert(name, MessageTransport::Success(Serialized::Json(value.to_owned())));
        }
        None => {
          return Err(Error::DeserializationError(format!(
            "Invalid port=value pair: '{}'",
            input
          )))
        }
      }
    }
    Ok(payload)
  }

  /// Remove a key from the held map and attempt to deserialize it into the destination type
  pub fn consume<T: DeserializeOwned>(&mut self, key: &str) -> Result<T> {
    let v = self
      .0
      .remove(key)
      .ok_or_else(|| Error::DeserializationError(format!("TransportMap does not have field '{}'", key)))?;
    let e = Err(Error::DeserializationError(format!(
      "Payload could not be converted to destination type. Payload was: {:?}",
      v
    )));
    match v {
      MessageTransport::Success(success) => match success {
        Serialized::MessagePack(bytes) => messagepack::deserialize(&bytes).map_err(de_err),
        Serialized::Struct(v) => raw::deserialize(v).map_err(de_err),
        Serialized::Json(v) => json::deserialize(&v).map_err(de_err),
      },
      MessageTransport::Failure(_) => e,
      MessageTransport::Signal(_) => e,
    }
  }

  /// Transpose any ports named "output" to "input". This is for a better user experience when
  /// trying to pipe components together without a full runtime. This should never be done
  /// without also providing a way to turn it off.
  #[doc(hidden)]
  pub fn transpose_output_name(&mut self) {
    let output = self.0.remove("output");
    if let Some(msg) = output {
      debug!("Transposing [output] to [input]");
      self.0.insert("input".to_owned(), msg);
    }
  }

  /// Returns true if any of the held messages is an error or exception type.
  #[must_use]
  pub fn has_error(&self) -> bool {
    for msg in self.0.values() {
      if msg.is_err() {
        return true;
      }
    }
    false
  }

  /// Returns an error if the transport is holding one, otherwise returns None.
  #[must_use]
  pub fn take_error(self) -> Option<MessageTransport> {
    for (_, v) in self.0 {
      if v.is_err() {
        return Some(v);
      }
    }
    None
  }

  /// Merge another [TransportMap] into the calling map.
  pub fn merge(&mut self, map: TransportMap) {
    for (k, v) in map.into_inner() {
      self.insert(k, v);
    }
  }
}

pub struct IntoIter {
  iter: Box<dyn Iterator<Item = TransportWrapper> + Send>,
}

impl IntoIter {
  pub fn new(map: TransportMap) -> Self {
    let iter = map.into_inner().into_iter().map(TransportWrapper::from);
    Self { iter: Box::new(iter) }
  }
}

impl Iterator for IntoIter {
  type Item = TransportWrapper;

  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next()
  }
}

impl std::fmt::Debug for IntoIter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("IntoIter<T>").finish()
  }
}

impl IntoIterator for TransportMap {
  type Item = TransportWrapper;

  type IntoIter = IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    IntoIter::new(self)
  }
}

impl From<wasmflow_packet::PacketMap> for TransportMap {
  fn from(map: wasmflow_packet::PacketMap) -> Self {
    let mut tmap = TransportMap::default();
    for (k, v) in map {
      tmap.insert(k, v.into());
    }
    tmap
  }
}

impl<K: AsRef<str>, V: Serialize> From<Vec<(K, V)>> for TransportMap {
  fn from(list: Vec<(K, V)>) -> Self {
    let mut map = TransportMap::default();
    for (k, v) in list {
      map.insert(k.as_ref(), MessageTransport::success(&v));
    }
    map
  }
}

impl<K: AsRef<str>, V: Serialize> From<HashMap<K, V>> for TransportMap {
  fn from(hashmap: HashMap<K, V>) -> Self {
    let mut map = TransportMap::default();
    for (k, v) in hashmap {
      map.insert(k.as_ref(), MessageTransport::success(&v));
    }
    map
  }
}

impl FromIterator<(String, MessageTransport)> for TransportMap {
  fn from_iter<T: IntoIterator<Item = (String, MessageTransport)>>(iter: T) -> Self {
    let mut map = TransportMap::default();
    for (k, v) in iter {
      map.insert(k, v);
    }
    map
  }
}

impl From<Vec<TransportWrapper>> for TransportMap {
  fn from(list: Vec<TransportWrapper>) -> Self {
    let mut map = TransportMap::default();
    for item in list {
      map.insert(item.port, item.payload);
    }
    map
  }
}

impl<V, const N: usize> From<[(&str, V); N]> for TransportMap
where
  V: Serialize + Sized,
{
  fn from(list: [(&str, V); N]) -> Self {
    let map: HashMap<String, V> = list.into_iter().map(|(key, val)| ((*key).to_owned(), val)).collect();
    map.into()
  }
}

fn de_err<T: Display>(e: T) -> Error {
  Error::DeserializationError(e.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::MessageTransport;

  #[test_log::test]
  fn test_merge() -> Result<()> {
    let mut map1 = TransportMap::default();
    map1.insert("first", MessageTransport::success(&"first-val"));
    let mut map2 = TransportMap::default();
    map2.insert("second", MessageTransport::success(&"second-val"));
    map1.merge(map2);
    let val1: String = map1.consume("first")?;
    assert_eq!(val1, "first-val");
    let val2: String = map1.consume("second")?;
    assert_eq!(val2, "second-val");

    Ok(())
  }
}
