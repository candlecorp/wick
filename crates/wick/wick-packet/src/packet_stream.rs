use std::pin::Pin;
use std::task::Poll;

use pin_project_lite::pin_project;
use tokio_stream::Stream;
use tracing::{span_enabled, Span};
use wasmrs_rx::FluxChannel;

use crate::{ContextTransport, InherentData, Packet, PacketExt, Result, RuntimeConfig};

pub type PacketSender = FluxChannel<Packet, crate::Error>;

type ContextConfig = (RuntimeConfig, InherentData);

#[cfg(target_family = "wasm")]
/// A Pin<Box<Stream>> of `T`.
pub type BoxStream<T> = Pin<Box<dyn Stream<Item = T> + 'static>>;
#[cfg(not(target_family = "wasm"))]
/// A Pin<Box<Stream>> of `T`.
pub type BoxStream<T> = Pin<Box<dyn Stream<Item = T> + Send + 'static>>;

#[cfg(target_family = "wasm")]
pin_project! {
  /// A stream of [Packet]s
  #[must_use]
  pub struct PacketStream {
    #[pin]
    inner: Box<dyn Stream<Item = Result<Packet>> + Unpin>,
    config: Option<ContextConfig>,
    span: Span
  }
}

#[cfg(not(target_family = "wasm"))]
pin_project! {
  /// A stream of [Packet]s
  #[must_use]
  pub struct PacketStream {
    #[pin]
    inner: Box<dyn Stream<Item = Result<Packet>> + Send + Unpin>,
    config: Option<ContextConfig>,
    span: Span
  }
}

impl Default for PacketStream {
  fn default() -> Self {
    PacketStream::empty()
  }
}

impl From<BoxStream<Result<Packet>>> for PacketStream {
  fn from(stream: BoxStream<Result<Packet>>) -> Self {
    Self::new(stream)
  }
}

impl From<Vec<Packet>> for PacketStream {
  fn from(iter: Vec<Packet>) -> Self {
    Self::new(Box::new(tokio_stream::iter(iter.into_iter().map(Ok))))
  }
}

impl PacketStream {
  #[cfg(target_family = "wasm")]
  pub fn new(rx: impl Stream<Item = Result<Packet>> + Unpin + 'static) -> Self {
    Self {
      inner: Box::new(tokio_stream::StreamExt::fuse(rx)),
      config: Default::default(),
      span: Span::current(),
    }
  }

  #[cfg(not(target_family = "wasm"))]
  pub fn new<T: Stream<Item = Result<Packet>> + Unpin + Send + 'static>(rx: T) -> Self {
    use tokio_stream::StreamExt;

    Self {
      inner: Box::new(rx.fuse()),
      config: Default::default(),
      span: Span::current(),
    }
  }

  pub fn noop() -> Self {
    Self::new(Box::new(tokio_stream::once(Ok(Packet::no_input()))))
  }

  pub fn set_span(&mut self, span: Span) {
    self.span = span;
  }

  pub fn set_context(&mut self, context: RuntimeConfig, inherent: InherentData) {
    self.config.replace((context, inherent));
  }

  pub fn new_channels() -> (PacketSender, Self) {
    let (flux, rx) = FluxChannel::new_parts();
    (flux, Self::new(Box::new(rx)))
  }

  pub fn empty() -> Self {
    Self::new(Box::new(tokio_stream::empty()))
  }
}

impl std::fmt::Debug for PacketStream {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("PacketStream").finish()
  }
}

impl Stream for PacketStream {
  type Item = Result<Packet>;

  fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
    let mut this = self;
    let mut this = Pin::new(&mut this);
    let config = this.config.take();
    let poll = { Pin::new(&mut *this.inner).poll_next(cx) };

    // Backwards compatibility note:
    // This is a hack added when context & operation configuration was introduced.
    // Rather than send it as a beginning packet, it's added as a sidecar to an existing packet and new
    // components expect it to exist in the first packet they receive.
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
            this.span.in_scope(|| {
              if span_enabled!(tracing::Level::TRACE) {
                let debug_packet = packet
                  .clone()
                  .decode_value()
                  .map_or_else(|_| format!("{:?}", packet.payload()), |j| j.to_string());
                let until = std::cmp::min(debug_packet.len(), 2048);
                this.span.in_scope(|| {
                  tracing::trace!(flags=packet.flags(), port=packet.port(), packet=%&debug_packet[..until], "packet");
                });
              }
            });
          }
          Poll::Ready(Some(Ok(packet)))
        }
        x => {
          this.config.replace(config);
          x
        }
      }
    } else {
      if let Poll::Ready(Some(Ok(packet))) = &poll {
        if cfg!(debug_assertions) {
          this.span.in_scope(|| {
            if span_enabled!(tracing::Level::TRACE) {
              let debug_packet = packet
                .clone()
                .decode_value()
                .map_or_else(|_| format!("{:?}", packet.payload()), |j| j.to_string());
              let until = std::cmp::min(debug_packet.len(), 2048);
              this.span.in_scope(|| {
                tracing::trace!(flags=packet.flags(), port=packet.port(), packet=%&debug_packet[..until], "packet");
              });
            }
          });
        }
      }
      poll
    }
  }
}

pub fn into_packet<N: Into<String>, T: serde::Serialize>(
  name: N,
) -> Box<dyn FnMut(anyhow::Result<T>) -> Result<Packet>> {
  let name = name.into();
  Box::new(move |x| Ok(x.map_or_else(|e| Packet::err(&name, e.to_string()), |x| Packet::encode(&name, &x))))
}
