use std::collections::HashMap;
use std::pin::Pin;

use futures::{Stream, StreamExt};
use pin_project_lite::pin_project;
use vino_transport::TransportWrapper;
use wasmflow_packet::{Packet, PacketWrapper};

pin_project! {
  /// A stream of [PacketWrapper]s
  #[must_use]
  pub struct PacketStream {
    finished: bool,
    buffer: Option<HashMap<String, Vec<Packet>>>,
    addons: Vec<PacketWrapper>,
    #[pin]
    stream: Box<dyn Stream<Item = PacketWrapper> + Unpin + Send + Sync>,
  }
}

impl std::fmt::Debug for PacketStream {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("PacketStream").finish()
  }
}

impl PacketStream {
  /// Instantiate a new [PacketStream]
  pub fn new(stream: Box<dyn Stream<Item = PacketWrapper> + Unpin + Send + Sync>) -> Self {
    Self {
      stream,
      buffer: None,
      addons: Vec::new(),
      finished: false,
    }
  }

  /// Manually add a packet to the stream.
  pub fn push(&mut self, packet: PacketWrapper) {
    self.addons.push(packet);
  }

  async fn buffer(&mut self) {
    if !self.finished {
      let mut map = HashMap::new();
      while let Some(next) = self.stream.next().await {
        let entry = map.entry(next.port).or_insert_with(Vec::<Packet>::new);
        if !next.payload.is_signal() {
          entry.push(next.payload);
        }
      }
      self.buffer = Some(map);
      self.finished = true;
    }
  }

  /// Wait for a packet stream to complete and return all packets for the specified port.
  pub async fn take_port(&mut self, port: &str) -> Option<Vec<Packet>> {
    self.buffer().await;
    let buffer = self.buffer.as_mut().unwrap();
    buffer.remove(port)
  }

  /// Wait for a packet stream to complete and return the packets mapped to their output port.
  pub async fn as_map(&mut self) -> Result<HashMap<String, Vec<Packet>>, crate::error::Error> {
    if self.finished {
      Err(crate::error::Error::Closed)
    } else {
      self.buffer().await;
      self.buffer.take().ok_or(crate::error::Error::Closed)
    }
  }
}

impl Stream for PacketStream {
  type Item = PacketWrapper;

  fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
    let this = self.project();
    if !this.addons.is_empty() {
      let packet = this.addons.pop().unwrap();
      std::task::Poll::Ready(Some(packet))
    } else {
      this.stream.poll_next(cx)
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let (a, b) = self.stream.size_hint();
    (a, b.map(|v| v + self.addons.len()))
  }
}

pin_project! {
  /// A stream of [PacketWrapper]s
  #[must_use]
  pub struct TransportStream2 {
    #[pin]
    stream: Box<dyn Stream<Item = TransportWrapper> + Unpin + Send + Sync>,
  }
}

impl std::fmt::Debug for TransportStream2 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("PacketStream").finish()
  }
}

impl TransportStream2 {
  /// Instantiate a new [TransportStream2]
  pub fn new(stream: Box<dyn Stream<Item = TransportWrapper> + Unpin + Send + Sync>) -> Self {
    Self { stream }
  }
}

impl Stream for TransportStream2 {
  type Item = TransportWrapper;

  fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
    let this = self.project();

    this.stream.poll_next(cx)
  }
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.stream.size_hint()
  }
}
