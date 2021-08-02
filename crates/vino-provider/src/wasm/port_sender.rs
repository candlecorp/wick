use serde::Serialize;
use vino_component::v0;

use super::{
  port_close,
  port_send,
  port_send_close,
  CallResult,
};

pub trait PortSender {
  type Output: Serialize;
  fn send(&self, payload: &Self::Output) -> CallResult {
    port_send(
      &self.get_invocation_id(),
      &self.get_name(),
      v0::Payload::to_messagepack(payload),
    )
  }
  fn done(&self, payload: &Self::Output) -> CallResult {
    port_send_close(
      &self.get_invocation_id(),
      &self.get_name(),
      v0::Payload::to_messagepack(payload),
    )
  }
  fn exception(&self, message: String) -> CallResult {
    port_send(
      &self.get_invocation_id(),
      &self.get_name(),
      v0::Payload::Exception(message),
    )
  }
  fn done_exception(&self, message: String) -> CallResult {
    port_send_close(
      &self.get_invocation_id(),
      &self.get_name(),
      v0::Payload::Exception(message),
    )
  }
  fn close(&self) -> CallResult {
    port_close(&self.get_invocation_id(), &self.get_name())
  }
  fn get_invocation_id(&self) -> String;
  fn get_name(&self) -> String;
}
