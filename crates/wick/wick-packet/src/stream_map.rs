use std::collections::HashMap;

use tokio_stream::StreamExt;

use crate::{Error, Packet, PacketSender, PacketStream};
pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
#[must_use]
/// A wrapper for a map of [String]s to [PacketStream]s.
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

  /// Get the keys in the map.
  pub fn keys(&self) -> impl Iterator<Item = &String> {
    self.0.keys()
  }

  /// Take the next packet from the stream keyed by [key].
  pub async fn next_for(&mut self, key: &str) -> Option<Result<Packet>> {
    let stream = self.0.get_mut(key)?;
    stream.next().await
  }

  /// Take one packet from each stream in the map. Returns an error if a complete set can't be made.
  pub async fn next_set(&mut self) -> Result<Option<HashMap<String, Packet>>> {
    let keys = self.0.keys().cloned().collect::<Vec<_>>();
    let mut raw = HashMap::new();
    for key in keys {
      let packet = self.next_for(&key).await;
      raw.insert(key, packet);
    }
    if raw.values().all(|v| v.is_none()) {
      Ok(None)
    } else if let Some((name, _)) = raw.iter().find(|(_, p)| p.is_none()) {
      Err(Error::StreamMapMissing(name.clone()))
    } else {
      let mut rv = HashMap::new();
      for (key, packet) in raw {
        let packet = packet.unwrap();
        if let Err(e) = &packet {
          return Err(Error::StreamMapError(key, e.to_string()));
        }

        rv.insert(key, packet.unwrap());
      }
      Ok(Some(rv))
    }
  }

  #[cfg(feature = "rt-tokio")]
  /// Turn a single [PacketStream] into a [StreamMap] keyed by the passed [ports].
  pub fn from_stream(mut stream: PacketStream, ports: impl IntoIterator<Item = String>) -> Self {
    use wasmrs_rx::Observer;

    #[must_use]
    let mut streams = StreamMap::default();
    let mut senders = HashMap::new();
    for port in ports {
      senders.insert(port.clone(), streams.init(&port));
    }
    tokio::spawn(async move {
      while let Some(Ok(packet)) = stream.next().await {
        if packet.is_fatal_error() {
          for (name, sender) in senders.iter_mut() {
            let _ = sender.send(packet.clone().set_port(name));
          }
        } else {
          let sender = senders.get_mut(packet.port()).unwrap();
          let is_done = packet.is_done();
          let _ = sender.send(packet);
          if is_done {
            sender.complete();
          }
        }
      }
    });
    streams
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
