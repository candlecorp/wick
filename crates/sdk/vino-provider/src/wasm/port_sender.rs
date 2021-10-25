use serde::Serialize;
use vino_packet::v0;

use super::{
  port_close,
  port_send,
  port_send_close,
  Error,
};

/// The WebAssembly-based PortSender trait. This trait encapsulates sending messages out of a WebAssembly component's ports.
pub trait PortSender {
  /// The type of data that the port outputs.
  type PayloadType: Serialize;

  /// Send a message.
  fn send(&self, payload: &Self::PayloadType) -> Result<(), Error> {
    port_send(
      &self.get_name(),
      self.get_id(),
      v0::Payload::messagepack(payload),
    )
  }

  /// Send a message then close the port.
  fn done(&self, payload: &Self::PayloadType) -> Result<(), Error> {
    port_send_close(
      &self.get_name(),
      self.get_id(),
      v0::Payload::messagepack(payload),
    )
  }

  /// Send an exception.
  fn send_exception(&self, message: String) -> Result<(), Error> {
    port_send(
      &self.get_name(),
      self.get_id(),
      v0::Payload::Exception(message),
    )
  }

  /// Send an exception then close the port.
  fn done_exception(&self, message: String) -> Result<(), Error> {
    port_send_close(
      &self.get_name(),
      self.get_id(),
      v0::Payload::Exception(message),
    )
  }

  /// Signal that a job is finished with the port.
  fn close(&self) -> Result<(), Error> {
    port_close(&self.get_name(), self.get_id())
  }

  /// Get the name of the port.
  fn get_name(&self) -> String;

  /// Return the ID of the transaction.
  fn get_id(&self) -> u32;
}
