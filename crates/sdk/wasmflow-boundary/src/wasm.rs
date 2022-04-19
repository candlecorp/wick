use std::collections::HashMap;

use crate::incoming::IncomingPayload;

/// Convert a MessagePack-ed buffer into an [IncomingPayload].

pub fn from_buffer<C, S>(
  buffer: &[u8],
) -> Result<IncomingPayload<EncodedMap, C, S>, Box<dyn std::error::Error + Send + Sync>>
where
  C: std::fmt::Debug + serde::de::DeserializeOwned,
  S: std::fmt::Debug + serde::de::DeserializeOwned,
{
  let (id, payload, config, state): (u32, HashMap<String, Vec<u8>>, Option<C>, Option<S>) =
    wasmflow_codec::messagepack::deserialize(buffer)?;

  Ok(IncomingPayload::new(id, EncodedMap(payload), config, state))
}

/// A map of port names to MessagePack encoded [Packet]s.
#[derive(Debug, serde::Deserialize)]
#[serde(transparent)]
pub struct EncodedMap(HashMap<String, Vec<u8>>);

impl EncodedMap {
  /// Get the contained bytes for the specified port.
  pub fn get(&self, port: &str) -> Result<&Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    self
      .0
      .get(port)
      .ok_or_else(|| format!("Attempted to take packet from port '{}' that had no data.", port).into())
  }
}
