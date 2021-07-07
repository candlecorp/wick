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
use vino_component::Packet;

// TODO: get this out of here
#[doc(hidden)]
pub trait Sender {
  /// The type of data that the port outputs
  type PayloadType: Serialize + Send + 'static;

  /// Get the port buffer that the sender can push to
  fn get_port(&self) -> Arc<Mutex<Port>>;

  /// Buffer a message
  fn send(&self, data: Self::PayloadType) {
    self.push(Packet::V0(ComponentPayload::to_messagepack(data)));
  }

  /// Buffer a message then close the port
  fn done(&self, data: Self::PayloadType) {
    self.send(data);
    self.send_message(Packet::V0(ComponentPayload::Close));
  }

  /// Buffer a complete Output message then close the port
  fn push(&self, output: Packet) {
    let port = self.get_port();
    let mut port = port.lock().unwrap();
    port.buffer.push_back(output);
  }

  /// Buffer a payload
  fn send_message(&self, packet: Packet) {
    let port = self.get_port();
    let mut port = port.lock().unwrap();
    port.buffer.push_back(packet);
  }

  /// Buffer a payload then close the port
  fn done_message(&self, packet: Packet) {
    self.send_message(packet);
    self.send_message(Packet::V0(ComponentPayload::Close));
  }

  /// Buffer an exception
  fn send_exception(&self, payload: String) {
    let port = self.get_port();
    let mut port = port.lock().unwrap();
    port
      .buffer
      .push_back(Packet::V0(ComponentPayload::Error(payload)));
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

// TODO: get this out of here
#[doc(hidden)]
#[must_use]
#[derive(Debug, Clone)]
pub struct Port {
  name: String,
  buffer: VecDeque<Packet>,
  status: PortStatus,
}

impl Port {
  #[doc(hidden)]
  pub fn new(name: String) -> Self {
    Self {
      name,
      buffer: VecDeque::new(),
      status: PortStatus::Open,
    }
  }
  #[doc(hidden)]
  #[must_use]
  pub fn is_closed(&self) -> bool {
    self.status == PortStatus::Closed
  }
  #[doc(hidden)]
  pub fn close(&mut self) {
    trace!("Port {} is closing", self.name);
    self.status = PortStatus::Closed;
  }
}

// TODO: get this out of here
#[doc(hidden)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PortStatus {
  Closed,
  Open,
}

// TODO: get this out of here
#[doc(hidden)]
#[must_use]
#[derive()]
pub struct PortStream {
  streams: Vec<Arc<Mutex<Port>>>,
}

impl std::fmt::Debug for PortStream {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Receiver")
      .field("streams", &self.streams.len())
      .finish()
  }
}

impl Clone for PortStream {
  fn clone(&self) -> Self {
    Self {
      streams: self.streams.clone(),
    }
  }
}

impl PortStream {
  #[doc(hidden)]
  pub fn new(buffer: Vec<Arc<Mutex<Port>>>) -> Self {
    Self { streams: buffer }
  }
}

#[doc(hidden)]
#[must_use]
#[derive(Debug, Clone)]
pub struct PortPacket {
  pub port: String,
  pub packet: Packet,
}

impl Stream for PortStream {
  type Item = PortPacket;

  fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    let mut all_closed = true;
    for port in self.streams.iter() {
      let mut port = port.lock().unwrap();
      if !port.buffer.is_empty() {
        return match port.buffer.pop_front() {
          Some(payload) => {
            if payload == Packet::V0(ComponentPayload::Close) {
              port.close();
            }
            Poll::Ready(Some(PortPacket {
              port: port.name.clone(),
              packet: payload,
            }))
          }
          None => unreachable!(),
        };
      } else if port.is_closed() {
        all_closed = all_closed && port.is_closed();
      }
    }
    if all_closed {
      trace!("Port stream is shutting down");
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
