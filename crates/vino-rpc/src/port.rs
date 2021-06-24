use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::{
  Arc,
  Mutex,
};
use std::task::{
  Context,
  Poll,
};

use futures::Stream;
use serde::Serialize;
use vino_component::v0::Payload as ComponentPayload;
use vino_component::Output;

pub trait Sender {
  /// The type of data that the port outputs
  type PayloadType: Serialize + Send + 'static;

  /// Get the port buffer that the sender can push to
  fn get_port(&self) -> Arc<Mutex<Port>>;

  /// Buffer a message
  fn send(&self, data: Self::PayloadType) {
    self.push(Output::V0(ComponentPayload::to_messagepack(data)));
  }

  /// Buffer a message then close the port
  fn done(&self, data: Self::PayloadType) {
    self.send(data);
    self.close();
  }

  /// Buffer a complete Output message then close the port
  fn push(&self, output: Output) {
    let port = self.get_port();
    let mut port = port.lock().unwrap();
    port.buffer.push_back(output);
  }

  /// Buffer a payload
  fn send_message(&self, payload: ComponentPayload) {
    let port = self.get_port();
    let mut port = port.lock().unwrap();
    port.buffer.push_back(Output::V0(payload));
  }

  /// Buffer a payload then close the port
  fn done_message(&self, payload: ComponentPayload) {
    self.send_message(payload);
    self.close();
  }

  /// Buffer an exception
  fn send_exception(&self, payload: String) {
    let port = self.get_port();
    let mut port = port.lock().unwrap();
    port
      .buffer
      .push_back(Output::V0(ComponentPayload::Error(payload)));
  }

  /// Buffer an exception then close the port
  fn done_exception(&self, payload: String) {
    self.send_exception(payload);
    self.close();
  }
  fn close(&self) {
    let port = self.get_port();
    let mut port = port.lock().unwrap();
    port.close();
  }
}

pub struct Port {
  name: String,
  buffer: VecDeque<Output>,
  status: PortStatus,
}

impl Port {
  pub fn new(name: &str) -> Self {
    Self {
      name: name.to_string(),
      buffer: VecDeque::new(),
      status: PortStatus::Open,
    }
  }
  pub fn is_closed(&self) -> bool {
    self.status == PortStatus::Closed
  }
  pub fn close(&mut self) {
    self.status = PortStatus::Closed;
  }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PortStatus {
  Closed,
  Open,
}

#[derive()]
pub struct Receiver {
  streams: Vec<Arc<Mutex<Port>>>,
}

impl std::fmt::Debug for Receiver {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Receiver")
      .field("streams", &self.streams.len())
      .finish()
  }
}

impl Clone for Receiver {
  fn clone(&self) -> Self {
    Self {
      streams: self.streams.clone(),
    }
  }
}

impl Receiver {
  pub fn new(buffer: Vec<Arc<Mutex<Port>>>) -> Self {
    Self { streams: buffer }
  }
}

impl Stream for Receiver {
  type Item = (String, Output);

  fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    let mut all_closed = true;
    for port in self.streams.iter() {
      let mut port = port.lock().unwrap();
      if !port.buffer.is_empty() {
        return match port.buffer.pop_front() {
          Some(payload) => Poll::Ready(Some((port.name.clone(), payload))),
          None => unreachable!(),
        };
      } else if port.is_closed() {
        all_closed = all_closed && port.is_closed();
      }
    }
    if all_closed {
      Poll::Ready(None)
    } else {
      Poll::Pending
    }
  }
}

// #[cfg(test)]
// mod tests {

//   use crate::Result;

//   #[test_env_log::test(tokio::test)]
//   async fn test() -> Result<()> {
//     Ok(())
//   }
// }
