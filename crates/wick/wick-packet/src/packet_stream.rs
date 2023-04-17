use std::pin::Pin;
use std::task::Poll;

use futures::Stream;
use pin_project_lite::pin_project;
use wasmrs_rx::{FluxChannel, Observer};

use crate::Packet;

pub type PacketSender = FluxChannel<Packet, crate::Error>;

#[cfg(target_family = "wasm")]
pin_project! {
  /// A stream of [Packet]s
  #[must_use]
  pub struct PacketStream {
    #[pin]
    inner: std::sync::Arc<parking_lot::Mutex<dyn Stream<Item = Result<Packet, crate::Error>> + Unpin>>,
  }
}

#[cfg(not(target_family = "wasm"))]
pin_project! {
  /// A stream of [Packet]s
  #[must_use]
  pub struct PacketStream {
    #[pin]
    inner: std::sync::Arc<parking_lot::Mutex<dyn Stream<Item = Result<Packet, crate::Error>> + Send + Unpin>>,
  }
}

impl From<Vec<Packet>> for PacketStream {
  fn from(iter: Vec<Packet>) -> Self {
    Self::new(Box::new(futures::stream::iter(iter.into_iter().map(Ok))))
  }
}

impl PacketStream {
  #[cfg(target_family = "wasm")]
  pub fn new(rx: Box<dyn Stream<Item = Result<Packet, crate::Error>> + Unpin>) -> Self {
    Self {
      inner: std::sync::Arc::new(parking_lot::Mutex::new(rx)),
    }
  }
  #[cfg(not(target_family = "wasm"))]
  pub fn new(rx: impl Stream<Item = Result<Packet, crate::Error>> + Unpin + Send + 'static) -> Self {
    Self {
      inner: std::sync::Arc::new(parking_lot::Mutex::new(rx)),
    }
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

  fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
    let mut stream = self.inner.lock();

    Pin::new(&mut *stream).poll_next(cx)
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;

  fn is_sync_send<T>()
  where
    T: Send + Sync,
  {
  }

  #[test]
  fn test_sync_send() -> Result<()> {
    is_sync_send::<PacketStream>();
    Ok(())
  }
}
