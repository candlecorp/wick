use parking_lot::Mutex;
use wasmflow_schematic_graph::{PortDirection, PortReference};
use wasmflow_sdk::v1::transport::{MessageSignal, MessageTransport, TransportWrapper};

use super::port_buffer::PortBuffer;
use super::PortStatus;
use crate::graph::types::ComponentPort;
use crate::ExecutionError;
type Result<T> = std::result::Result<T, ExecutionError>;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BufferAction {
  Consumed(TransportWrapper),
  Buffered,
}

impl std::fmt::Display for BufferAction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        BufferAction::Consumed(_) => "consumed",
        BufferAction::Buffered => "buffered",
      }
    )
  }
}

#[derive()]
#[must_use]
pub(crate) struct PortHandler {
  name: String,
  buffer: PortBuffer,
  status: Mutex<PortStatus>,
  port: ComponentPort,
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
  pub(super) fn new(port: ComponentPort) -> Self {
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

  pub(super) fn buffer(&self, value: TransportWrapper) -> Result<BufferAction> {
    assert!(
      self.get_status() != PortStatus::DoneClosed,
      "port should never be pushed to after being closed."
    );

    let action = if value.payload == MessageTransport::Signal(MessageSignal::Done) {
      let action = match self.port.direction() {
        PortDirection::In if !self.port.is_graph_output() => BufferAction::Consumed(value),
        PortDirection::In | PortDirection::Out => {
          self.buffer.push(value);
          BufferAction::Buffered
        }
      };
      if !self.is_empty() {
        self.set_status(PortStatus::DoneClosing);
      } else {
        self.set_status(PortStatus::DoneClosed);
      }
      action
    } else {
      self.buffer.push(value);
      BufferAction::Buffered
    };
    trace!(%action, "incoming message");
    Ok(action)
  }

  pub(super) fn take(&self) -> Option<TransportWrapper> {
    let result = self.buffer.take();
    debug!(port=%self.port,payload=?result, "taking message from buffer");

    let status = self.get_status();
    if self.is_empty() && status == PortStatus::DoneClosing {
      self.set_status(PortStatus::DoneClosed);
    }
    result
  }

  pub(crate) fn is_empty(&self) -> bool {
    self.buffer.is_empty()
  }

  pub(crate) fn len(&self) -> usize {
    self.buffer.len()
  }

  pub(crate) fn clone_buffer(&self) -> Vec<TransportWrapper> {
    self.buffer.clone_buffer()
  }
}
