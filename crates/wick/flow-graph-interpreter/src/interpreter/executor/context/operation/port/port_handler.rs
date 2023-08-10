use std::ops::RangeBounds;

use flow_graph::PortReference;
use parking_lot::Mutex;
use tracing::Span;
use wick_packet::Packet;

use super::port_buffer::PortBuffer;
use super::PortStatus;
use crate::graph::types::OperationPort;

type PacketType = Packet;

#[derive()]
#[must_use]
pub(crate) struct PortHandler {
  operation_instance: String,
  buffer: PortBuffer,
  status: Mutex<PortStatus>,
  port: OperationPort,
}

impl std::fmt::Display for PortHandler {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}.{} ({}, {})",
      self.operation_instance,
      self.port,
      self.port.direction(),
      self.get_status()
    )
  }
}

impl std::fmt::Debug for PortHandler {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("PortHandler")
      .field("port", &self.port)
      .field("buffer", &self.buffer)
      .field("status", &self.status)
      .finish()
  }
}

impl PortHandler {
  pub(super) fn new(operation_instance: impl AsRef<str>, port: OperationPort) -> Self {
    Self {
      buffer: Default::default(),
      operation_instance: operation_instance.as_ref().to_owned(),
      port,
      status: Mutex::new(PortStatus::Open),
    }
  }

  pub(crate) fn status(&self) -> PortStatus {
    let lock = self.status.lock();
    *lock
  }

  pub(crate) fn set_status(&self, new_status: PortStatus) {
    let new_status =
      if new_status == PortStatus::DoneClosed && !self.is_empty() && self.status() != PortStatus::DoneClosed {
        PortStatus::DoneClosing
      } else {
        new_status
      };

    let curr_status = self.get_status();

    if curr_status != new_status {
      Span::current().in_scope(|| {
        if curr_status == PortStatus::DoneClosed && new_status != PortStatus::DoneClosed {
          debug!(
            op = %self.operation_instance,
            port = %self.port,
            dir  = %self.port.direction(),
            from = %curr_status,
            to = %new_status,
            "trying to set new status on closed port");
        } else {
          trace!(
            op = %self.operation_instance,
            port = %self.port,
            dir  = %self.port.direction(),
            from = %curr_status,
            to = %new_status,
            "setting port status");

          *self.status.lock() = new_status;
        }
      });
    }
  }

  pub(crate) fn port_ref(&self) -> PortReference {
    self.port.detached()
  }

  pub(crate) fn name(&self) -> &str {
    self.port.name()
  }

  pub(crate) fn get_status(&self) -> PortStatus {
    *self.status.lock()
  }

  pub(super) fn buffer(&self, value: PacketType) {
    if self.get_status() == PortStatus::DoneClosed {
      warn!(port=%self, "trying to buffer on closed port");
    }
    if value.is_done() {
      self.set_status(PortStatus::DoneClosing);
    }
    self.buffer.push(value);
  }

  pub(super) fn take(&self) -> Option<PacketType> {
    let result = self.buffer.take();

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
    trace!(port=%self,packets=?packets, "draining buffer");

    let status = self.get_status();
    if self.is_empty() && status == PortStatus::DoneClosing {
      self.set_status(PortStatus::DoneClosed);
    }
    packets
  }

  pub(crate) fn is_empty(&self) -> bool {
    self.buffer.is_empty()
  }
}
