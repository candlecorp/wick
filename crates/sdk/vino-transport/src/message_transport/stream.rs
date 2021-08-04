use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::pin::Pin;
use std::task::Poll;

use tokio::sync::mpsc::UnboundedReceiver;
use tokio_stream::{
  Stream,
  StreamExt,
};

use super::{
  CLOSE_MESSAGE,
  SYSTEM_CLOSE_MESSAGE,
};
use crate::TransportWrapper;

/// A [MessageTransportStream] is a stream of [MessageTransport]s
#[derive(Debug)]
pub struct MessageTransportStream {
  rx: RefCell<UnboundedReceiver<TransportWrapper>>,
  buffer: HashMap<String, Vec<TransportWrapper>>,
  collected: bool,
}

impl MessageTransportStream {
  #[doc(hidden)]
  #[must_use]
  pub fn new(rx: UnboundedReceiver<TransportWrapper>) -> Self {
    Self {
      rx: RefCell::new(rx),
      buffer: HashMap::new(),
      collected: false,
    }
  }
}

impl Stream for MessageTransportStream {
  type Item = TransportWrapper;

  fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
    let mut rx = self.rx.borrow_mut();
    match rx.poll_recv(cx) {
      Poll::Ready(Some(msg)) => {
        if SYSTEM_CLOSE_MESSAGE.eq(&msg) {
          Poll::Ready(None)
        } else {
          Poll::Ready(Some(msg))
        }
      }
      Poll::Ready(None) => Poll::Ready(None),
      Poll::Pending => Poll::Pending,
    }
  }
}

impl MessageTransportStream {
  /// Collect all the [TransportWrapper] items associated with the passed port.
  pub async fn collect_port<B: FromIterator<TransportWrapper>>(&mut self, port: &str) -> B {
    if !self.collected {
      let mut buffer = HashMap::new();
      self.collected = true;
      let all: Vec<_> = self.collect().await;
      for message in all {
        let buff = buffer.entry(message.port.clone()).or_insert_with(Vec::new);
        // If we've collected everything, we don't care about Close messages
        if CLOSE_MESSAGE.ne(&message.payload) {
          buff.push(message);
        }
      }
      self.buffer = buffer;
    }

    self
      .buffer
      .remove(port)
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
    tx.send(SYSTEM_CLOSE_MESSAGE.clone())?;
    let mut stream = MessageTransportStream::new(rx);

    let a_msgs: Vec<_> = stream.collect_port("A").await;
    assert_eq!(stream.buffered_size(), (1, 2));
    let b_msgs: Vec<_> = stream.collect_port("B").await;
    assert_eq!(stream.buffered_size(), (0, 0));
    assert_eq!(a_msgs.len(), 2);
    assert_eq!(b_msgs.len(), 2);
    Ok(())
  }
}
