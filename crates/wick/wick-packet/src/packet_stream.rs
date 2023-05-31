use std::pin::Pin;
use std::task::Poll;

use futures::stream::{BoxStream, FusedStream};
use futures::Stream;
use pin_project_lite::pin_project;
use tracing::Span;
use wasmrs_rx::FluxChannel;

use crate::{ContextTransport, GenericConfig, InherentData, Packet};

pub type PacketSender = FluxChannel<Packet, crate::Error>;

type ContextConfig = (GenericConfig, Option<InherentData>);

#[cfg(target_family = "wasm")]
pin_project! {
  /// A stream of [Packet]s
  #[must_use]
  pub struct PacketStream {
    #[pin]
    inner: std::sync::Arc<parking_lot::Mutex<dyn FusedStream<Item = Result<Packet, crate::Error>> + Unpin>>,
    config: std::sync::Arc<parking_lot::Mutex<Option<ContextConfig>>>,
    span: Span
  }
}

#[cfg(not(target_family = "wasm"))]
pin_project! {
  /// A stream of [Packet]s
  #[must_use]
  pub struct PacketStream {
    #[pin]
    inner: std::sync::Arc<parking_lot::Mutex<dyn FusedStream<Item = Result<Packet, crate::Error>> + Send + Unpin>>,
    config: std::sync::Arc<parking_lot::Mutex<Option<ContextConfig>>>,
    span: Span
  }
}

impl From<BoxStream<'static, Result<Packet, crate::Error>>> for PacketStream {
  fn from(stream: BoxStream<'static, Result<Packet, crate::Error>>) -> Self {
    Self::new(stream)
  }
}

impl From<Vec<Packet>> for PacketStream {
  fn from(iter: Vec<Packet>) -> Self {
    Self::new(Box::new(futures::stream::iter(iter.into_iter().map(Ok))))
  }
}

impl PacketStream {
  #[cfg(target_family = "wasm")]
  pub fn new(rx: impl Stream<Item = Result<Packet, crate::Error>> + Unpin + 'static) -> Self {
    Self {
      inner: std::sync::Arc::new(parking_lot::Mutex::new(futures::StreamExt::fuse(rx))),
      config: Default::default(),
      span: Span::current(),
    }
  }
  #[cfg(not(target_family = "wasm"))]
  pub fn new(rx: impl Stream<Item = Result<Packet, crate::Error>> + Unpin + Send + 'static) -> Self {
    use futures::StreamExt;

    Self {
      inner: std::sync::Arc::new(parking_lot::Mutex::new(rx.fuse())),
      config: Default::default(),
      span: Span::current(),
    }
  }

  pub fn set_span(&mut self, span: Span) {
    self.span = span;
  }

  pub fn set_context(&self, context: GenericConfig, seed: Option<InherentData>) {
    self.config.lock().replace((context, seed));
  }

  pub fn new_channels() -> (PacketSender, Self) {
    let (flux, rx) = FluxChannel::new_parts();
    (flux, Self::new(Box::new(rx)))
  }

  pub fn empty() -> Self {
    Self::new(Box::new(futures::stream::empty()))
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
    let config = self.config.lock().take();
    let poll = {
      let mut stream = self.inner.lock();
      Pin::new(&mut *stream).poll_next(cx)
    };

    if let Some(config) = config {
      match poll {
        Poll::Ready(Some(Ok(mut packet))) => {
          packet.set_context(
            wasmrs_codec::messagepack::serialize(&ContextTransport::new(config.0, config.1))
              .unwrap()
              .into(),
          );
          tracing::trace!("attached context to packet on port '{}'", packet.port());
          if cfg!(debug_assertions) {
            self.span.in_scope(|| {
              tracing::trace!(flags=packet.flags(),port=packet.port(),packet=%packet.clone().deserialize_generic().map_or_else(|_| format!("{:?}", packet.payload()),|j|j.to_string())
              , "delivering packet");
            });
          }
          Poll::Ready(Some(Ok(packet)))
        }
        x => {
          self.config.lock().replace(config);
          x
        }
      }
    } else {
      if let Poll::Ready(Some(Ok(packet))) = &poll {
        if cfg!(debug_assertions) {
          self.span.in_scope(|| {
            tracing::trace!(flags=packet.flags(),port=packet.port(),packet=%packet.clone().deserialize_generic().map_or_else(|_| format!("{:?}", packet.payload()),|j|j.to_string())
              , "delivering packet");
          });
        }
      }
      poll
    }
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
