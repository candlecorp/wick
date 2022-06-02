use core::task::{Context, Poll};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;

use parking_lot::Mutex;
use tokio_stream::{Stream, StreamExt};
use wasmflow_packet::PacketWrapper;

use super::transport_wrapper::TransportWrapper;
use crate::{Error, MessageSignal, MessageTransport};

/// A [TransportStream] is a stream of [crate::TransportWrapper]s.
#[must_use]
pub struct TransportStream {
  rx: Mutex<Pin<Box<dyn Stream<Item = TransportWrapper> + Send>>>,
  buffer: HashMap<String, Vec<TransportWrapper>>,
  collected: bool,
  done: AtomicBool,
}

impl std::fmt::Debug for TransportStream {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TransportStream")
      .field("buffer", &self.buffer)
      .field("collected", &self.collected)
      .finish()
  }
}

impl TransportStream {
  /// Constructor for [TransportStream].
  pub fn new(rx: impl Stream<Item = TransportWrapper> + Send + 'static) -> Self {
    Self {
      rx: Mutex::new(Box::pin(rx)),
      buffer: HashMap::new(),
      collected: false,
      done: AtomicBool::new(false),
    }
  }

  /// Convert a packet stream into a [TransportStream]
  pub fn from_packetstream<T>(stream: T) -> Self
  where
    T: Stream<Item = PacketWrapper> + Send + 'static,
  {
    Self {
      rx: Mutex::new(Box::pin(stream.map(|pw| pw.into()))),
      buffer: HashMap::new(),
      collected: false,
      done: AtomicBool::new(false),
    }
  }
}

impl Stream for TransportStream {
  type Item = TransportWrapper;

  fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    let done = self.done.load(std::sync::atomic::Ordering::SeqCst);
    if done {
      return Poll::Ready(None);
    }
    let mut inner = self.rx.lock();
    let pinned = Pin::new(&mut *inner);
    let poll = pinned.poll_next(cx);
    drop(inner);
    match poll {
      Poll::Ready(Some(msg)) => {
        if msg.is_system_close() {
          Poll::Ready(None)
        } else {
          if msg.is_component_error() {
            // If it's a component-wide error then signal we're ready to finish.
            self.done.store(true, std::sync::atomic::Ordering::SeqCst);
          }
          Poll::Ready(Some(msg))
        }
      }
      Poll::Ready(None) => Poll::Ready(None),
      Poll::Pending => Poll::Pending,
    }
  }
}

impl TransportStream {
  /// Collect all the [TransportWrapper] items associated with the passed port.
  pub async fn drain_port(&mut self, port: &str) -> Result<Vec<TransportWrapper>, Error> {
    let close_message = MessageTransport::Signal(MessageSignal::Done);
    if !self.collected {
      let mut buffer = HashMap::new();
      self.collected = true;
      let all: Vec<_> = self.collect().await;
      for message in all {
        let buff = buffer.entry(message.port.clone()).or_insert_with(Vec::new);
        // If we've collected everything, we don't care about Close messages
        if close_message.ne(&message.payload) {
          buff.push(message);
        }
      }
      self.buffer = buffer;
    }

    self
      .buffer
      .remove(port)
      .ok_or_else(|| Error::Other(format!("Port not found or already drained '{}'", port)))
  }

  /// Collect all the [TransportWrapper] items in the stream.
  pub async fn drain(&mut self) -> Vec<TransportWrapper> {
    let messages: Vec<_> = if !self.collected {
      self.collected = true;
      self.filter(|message| !message.payload.is_signal()).collect().await
    } else {
      let mut messages = Vec::new();
      for (_, buffer) in self.buffer.drain() {
        for message in buffer {
          messages.push(message);
        }
      }
      messages
    };
    messages
  }

  /// Returns the buffered number of ports and total number of messages.
  pub fn buffered_size(&self) -> (u8, usize) {
    let mut num_keys = 0;
    let mut num_msgs = 0;
    for msgs in self.buffer.values() {
      num_keys += 1;
      num_msgs += msgs.len();
    }
    (num_keys, num_msgs)
  }
}

#[cfg(test)]
mod tests {

  use tokio::sync::mpsc::unbounded_channel;
  use tokio_stream::wrappers::UnboundedReceiverStream;

  fn is_sync_send<T: Sync + Send>() {}

  use super::*;
  use crate::MessageTransport;
  #[test]
  fn test_sync_send() {
    is_sync_send::<TransportStream>();
  }

  #[test_log::test(tokio::test)]
  async fn test() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, rx) = unbounded_channel();
    let message = MessageTransport::success(&String::from("Test"));

    tx.send(TransportWrapper::new("A", message.clone()))?;
    tx.send(TransportWrapper::new("B", message.clone()))?;
    tx.send(TransportWrapper::new("A", message.clone()))?;
    tx.send(TransportWrapper::new("B", message.clone()))?;
    tx.send(TransportWrapper::new_system_close())?;
    let mut stream = TransportStream::new(UnboundedReceiverStream::new(rx));

    let a_msgs = stream.drain_port("A").await?;
    assert_eq!(stream.buffered_size(), (1, 2));
    let b_msgs = stream.drain_port("B").await?;
    assert_eq!(stream.buffered_size(), (0, 0));
    assert_eq!(a_msgs.len(), 2);
    assert_eq!(b_msgs.len(), 2);
    Ok(())
  }
}
