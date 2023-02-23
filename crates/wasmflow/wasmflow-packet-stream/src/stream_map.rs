use std::collections::HashMap;

use crate::{PacketSender, PacketStream};
pub(crate) type Result<T> = std::result::Result<T, crate::Error>;

#[derive(Default)]
#[must_use]
/// A wrapper for a map of [String]s to [MessageTransport]
pub struct StreamMap(HashMap<String, PacketStream>);

impl std::fmt::Debug for StreamMap {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("StreamMap").field(&self.0.keys()).finish()
  }
}

impl StreamMap {
  /// Remove a stream from the map by key name.
  pub fn take(&mut self, key: &str) -> Result<PacketStream> {
    let v = self
      .0
      .remove(key)
      .ok_or_else(|| crate::Error::PortMissing(key.to_owned()))?;
    Ok(v)
  }

  pub fn init(&mut self, port: &str) -> PacketSender {
    let flux = PacketSender::default();
    self
      .0
      .insert(port.to_owned(), PacketStream::new(Box::new(flux.take_rx().unwrap())));
    flux
  }
}

impl IntoIterator for StreamMap {
  type Item = (String, PacketStream);

  type IntoIter = std::collections::hash_map::IntoIter<String, PacketStream>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
