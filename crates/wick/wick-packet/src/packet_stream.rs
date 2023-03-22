use std::pin::Pin;

use futures::Stream;
use parking_lot::Mutex;
use wasmrs_rx::{FluxChannel, Observer};

use crate::Packet;

pub type PacketSender = FluxChannel<Packet, crate::Error>;

/// A stream of [Packet]s
#[must_use]
pub struct PacketStream {
  #[cfg(target_family = "wasm")]
  inner: Mutex<Box<dyn Stream<Item = Result<Packet, crate::Error>> + Unpin>>,
  #[cfg(not(target_family = "wasm"))]
  inner: Mutex<Box<dyn Stream<Item = Result<Packet, crate::Error>> + Unpin + Send + Sync>>,
}

impl From<Vec<Packet>> for PacketStream {
  fn from(iter: Vec<Packet>) -> Self {
    Self::new(Box::new(futures::stream::iter(iter.into_iter().map(Ok))))
  }
}

impl PacketStream {
  #[cfg(target_family = "wasm")]
  pub fn new(rx: Box<dyn Stream<Item = Result<Packet, crate::Error>> + Unpin>) -> Self {
    Self { inner: Mutex::new(rx) }
  }
  #[cfg(not(target_family = "wasm"))]
  pub fn new(rx: Box<dyn Stream<Item = Result<Packet, crate::Error>> + Unpin + Send + Sync>) -> Self {
    Self { inner: Mutex::new(rx) }
  }

  pub fn new_channels() -> (PacketSender, Self) {
    let flux = FluxChannel::new();
    let rx = flux.take_rx().unwrap();
    (flux, Self::new(Box::new(rx)))
  }
}

impl Default for PacketStream {
  fn default() -> Self {
    let flux = FluxChannel::new();
    flux.complete();
    Self::new(Box::new(flux.take_rx().unwrap()))
  }
}

impl std::fmt::Debug for PacketStream {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("PacketStream").finish()
  }
}

impl Stream for PacketStream {
  type Item = Result<Packet, crate::Error>;

  fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
    cx.waker().wake_by_ref();
    let mut inner = self.inner.lock();
    let pinned = Pin::new(&mut *inner);
    pinned.poll_next(cx)
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;

  fn sync_send<T>()
  where
    T: Sync + Send,
  {
  }

  #[test]
  fn test_sync_send() -> Result<()> {
    sync_send::<PacketStream>();
    Ok(())
  }
}
