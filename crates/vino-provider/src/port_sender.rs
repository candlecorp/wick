use serde::Serialize;
use vino_packet::Packet;

#[derive(Debug, Clone, Copy)]
pub struct SenderError {}
impl std::fmt::Display for SenderError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Error sending to output channel")
  }
}
impl std::error::Error for SenderError {}

/// The port interface.
pub trait PortSender {
  /// The type of data that the port outputs.
  type PayloadType: Serialize + Send + 'static;

  /// Send a message.
  fn send(&self, data: &Self::PayloadType) -> Result<(), SenderError>;

  /// Send a message then close the port.
  fn done(&self, data: &Self::PayloadType) -> Result<(), SenderError>;

  /// Send an exception.
  fn send_exception(&self, payload: String) -> Result<(), SenderError>;

  /// Send an exception then close the port.
  fn done_exception(&self, payload: String) -> Result<(), SenderError>;

  fn close(&self) -> Result<(), SenderError>;
}

pub trait RawSender {
  /// Send a complete Output message then close the port.
  fn push(&self, output: Packet) -> Result<(), SenderError>;

  /// Send a payload.
  fn send_message(&self, packet: Packet) -> Result<(), SenderError>;

  /// Send a payload then close the port.
  fn done_message(&self, packet: Packet) -> Result<(), SenderError>;
}
