use core::task::{Context, Poll};
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::pin::Pin;

use tokio_stream::{Stream, StreamExt};

use super::transport_wrapper::TransportWrapper;
use crate::{MessageSignal, MessageTransport};

/// A boxed [Stream] that produces [TransportWrapper]s
pub type BoxedTransportStream = Pin<Box<dyn Stream<Item = TransportWrapper> + Send + 'static>>;

/// Converts a [Stream] of [TransportWrapper]s into a stream of [serde_json::Value]s, optionally omitting signals.
pub fn map_to_json(
  stream: impl Stream<Item = TransportWrapper>,
  print_signals: bool,
) -> impl Stream<Item = serde_json::Value> {
  stream.filter_map(move |wrapper| {
    if wrapper.payload.is_signal() && !print_signals {
      None
    } else {
      Some(wrapper.into_json())
    }
  })
}

/// A [TransportStream] is a stream of [crate::TransportWrapper]s.
pub struct TransportStream {
  rx: RefCell<Pin<Box<dyn Stream<Item = TransportWrapper> + Send>>>,
  buffer: HashMap<String, Vec<TransportWrapper>>,
  collected: bool,
  done: RefCell<bool>,
}

impl std::fmt::Debug for TransportStream {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TransportStream")
      .field("rx", &String::from("<Ignored>"))
      .field("buffer", &self.buffer)
      .field("collected", &self.collected)
      .finish()
  }
}

impl TransportStream {
  /// Constructor for [TransportStream].
  #[must_use]
  pub fn new(rx: impl Stream<Item = TransportWrapper> + Send + 'static) -> Self {
    Self {
      rx: RefCell::new(Box::pin(rx)),
      buffer: HashMap::new(),
      collected: false,
      done: RefCell::new(false),
    }
  }
}

impl Stream for TransportStream {
  type Item = TransportWrapper;

  fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    let mut done = self.done.borrow_mut();
    if *done {
      return Poll::Ready(None);
    }
    let mut inner = self.rx.borrow_mut();
    let pinned = Pin::new(&mut *inner);
    match pinned.poll_next(cx) {
      Poll::Ready(Some(msg)) => {
        if msg.is_system_close() {
          Poll::Ready(None)
        } else {
          if msg.is_component_error() {
            // If it's a component-wide error then signal we're ready to finish.
            *done = true;
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
  pub async fn collect_port<T: AsRef<str> + Send, B: FromIterator<TransportWrapper>>(
    &mut self,
    port: T,
  ) -> B {
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
      .remove(port.as_ref())
      .unwrap_or_else(Vec::new)
      .into_iter()
      .collect()
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

  use tokio::sync::mpsc::error::SendError;
  use tokio::sync::mpsc::unbounded_channel;
  use tokio_stream::wrappers::UnboundedReceiverStream;

  use super::*;
  use crate::MessageTransport;
  #[test_env_log::test(tokio::test)]
  async fn test() -> Result<(), SendError<TransportWrapper>> {
    let (tx, rx) = unbounded_channel();
    let message = MessageTransport::success(&String::from("Test"));

    tx.send(TransportWrapper::new("A", message.clone()))?;
    tx.send(TransportWrapper::new("B", message.clone()))?;
    tx.send(TransportWrapper::new("A", message.clone()))?;
    tx.send(TransportWrapper::new("B", message.clone()))?;
    tx.send(TransportWrapper::new_system_close())?;
    let mut stream = TransportStream::new(UnboundedReceiverStream::new(rx));

    let a_msgs: Vec<_> = stream.collect_port("A").await;
    assert_eq!(stream.buffered_size(), (1, 2));
    let b_msgs: Vec<_> = stream.collect_port("B").await;
    assert_eq!(stream.buffered_size(), (0, 0));
    assert_eq!(a_msgs.len(), 2);
    assert_eq!(b_msgs.len(), 2);
    Ok(())
  }
}
