use std::collections::{
  HashMap,
  VecDeque,
};
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
use vino_guest::OutputPayload;
use vino_transport::serialize;

pub trait Sender {
  type Payload: Serialize;

  fn get_port(&self) -> Arc<Mutex<Port>>;

  fn send(&self, payload: Self::Payload) {
    match serialize(payload) {
      Ok(payload) => self.push(OutputPayload::MessagePack(payload)),
      Err(e) => self.push(OutputPayload::Error(e.to_string())),
    }
  }
  fn done(&self, payload: Self::Payload) {
    self.send(payload);
    self.close();
  }

  fn push(&self, payload: OutputPayload) {
    let port = self.get_port();
    let mut port = port.lock().unwrap();
    port.buffer.push_back(payload);
  }

  fn send_message(&self, payload: OutputPayload) {
    let port = self.get_port();
    let mut port = port.lock().unwrap();
    port.buffer.push_back(payload);
  }
  fn done_message(&self, payload: OutputPayload) {
    self.send_message(payload);
    self.close();
  }
  fn send_exception(&self, payload: String) {
    let port = self.get_port();
    let mut port = port.lock().unwrap();
    port.buffer.push_back(OutputPayload::Exception(payload));
  }
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

#[derive(Debug, Clone)]
pub struct Port {
  name: String,
  buffer: VecDeque<OutputPayload>,
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

pub type OutputStreams = HashMap<String, Arc<Mutex<Port>>>;

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
  type Item = (String, OutputPayload);

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

#[cfg(test)]
mod tests {

  use crate::Result;

  #[test_env_log::test(tokio::test)]
  async fn test() -> Result<()> {
    Ok(())
  }
}
