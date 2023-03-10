use std::ops::RangeBounds;

use flow_graph::{PortDirection, PortReference};
use parking_lot::Mutex;
use wick_packet::Packet;

use super::port_buffer::PortBuffer;
use super::PortStatus;
use crate::graph::types::OperationPort;

type PacketType = Packet;

#[derive()]
#[must_use]
pub(crate) struct PortHandler {
  name: String,
  buffer: PortBuffer,
  status: Mutex<PortStatus>,
  port: OperationPort,
}

impl std::fmt::Display for PortHandler {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}[{}]", self.name, self.get_status())
  }
}

impl std::fmt::Debug for PortHandler {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("PortHandler")
      .field("name", &self.name)
      .field("buffer", &self.buffer)
      .field("status", &self.status)
      .finish()
  }
}

impl PortHandler {
  pub(super) fn new(port: OperationPort) -> Self {
    Self {
      buffer: Default::default(),
      name: port.name().to_owned(),
      port,
      status: Mutex::new(PortStatus::Open),
    }
  }

  pub(crate) fn status(&self) -> PortStatus {
    let lock = self.status.lock();
    *lock
  }

  pub(crate) fn set_status(&self, status: PortStatus) {
    let new_status = if status == PortStatus::DoneClosed && !self.is_empty() {
      PortStatus::DoneClosing
    } else {
      status
    };

    let curr_status = self.get_status();

    if curr_status != new_status {
      trace!(old_status=%curr_status, new_status=%new_status, port=%self.port, name =self.name(), "setting port status");
      assert!(
        !(curr_status == PortStatus::DoneClosed && status != PortStatus::DoneClosed),
        "trying to set new status on closed port"
      );
      *self.status.lock() = new_status;
    }
  }

  pub(crate) fn port_ref(&self) -> PortReference {
    self.port.detached()
  }

  pub(crate) fn name(&self) -> &str {
    &self.name
  }

  pub(crate) fn get_status(&self) -> PortStatus {
    *self.status.lock()
  }

  pub(super) fn buffer(&self, value: PacketType) {
    assert!(
      self.get_status() != PortStatus::DoneClosed,
      "port should never be pushed to after being closed."
    );

    if value.is_done() {
      match self.port.direction() {
        PortDirection::In | PortDirection::Out => {
          self.buffer.push(value);
        }
      };
      if !self.is_empty() {
        self.set_status(PortStatus::DoneClosing);
      } else {
        self.set_status(PortStatus::DoneClosed);
      }
    } else {
      self.buffer.push(value);
    };
  }

  pub(super) fn take(&self) -> Option<PacketType> {
    let result = self.buffer.take();
    trace!(port=%self.port,payload=?result, "taking from buffer");

    let status = self.get_status();
    if self.is_empty() && status == PortStatus::DoneClosing {
      self.set_status(PortStatus::DoneClosed);
    }
    result
  }

  pub(super) fn drain<R>(&self, range: R) -> Vec<PacketType>
  where
    R: RangeBounds<usize>,
  {
    if self.buffer.is_empty() {
      return vec![];
    }
    let packets = self.buffer.drain(range);
    trace!(port=%self.port,packets=?packets, "taking from buffer");

    let status = self.get_status();
    if self.is_empty() && status == PortStatus::DoneClosing {
      self.set_status(PortStatus::DoneClosed);
    }
    packets
  }

  pub(crate) fn is_empty(&self) -> bool {
    self.buffer.is_empty()
  }

  // pub(crate) fn len(&self) -> usize {
  //   self.buffer.len()
  // }

  // pub(crate) fn clone_buffer(&self) -> Vec<PacketType> {
  //   self.buffer.clone_buffer()
  // }
}
