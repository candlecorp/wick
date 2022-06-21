use std::collections::VecDeque;
use std::fmt::Debug;

use parking_lot::Mutex;
use wasmflow_sdk::v1::transport::TransportWrapper;

type PacketType = TransportWrapper;

pub(super) struct PortBuffer {
  buffer: Mutex<VecDeque<PacketType>>,
}

impl Debug for PortBuffer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let lock = self.buffer.lock();
    let len = lock.len();
    f.debug_struct("Buffer").field("len", &len).finish()
  }
}

impl Default for PortBuffer {
  fn default() -> Self {
    Self {
      buffer: Mutex::new(Default::default()),
    }
  }
}

impl PortBuffer {
  pub(super) fn push(&self, value: PacketType) {
    let mut lock = self.buffer.lock();
    lock.push_back(value);
  }

  pub(super) fn is_empty(&self) -> bool {
    let lock = self.buffer.lock();
    lock.is_empty()
  }

  pub(super) fn len(&self) -> usize {
    let lock = self.buffer.lock();
    lock.len()
  }

  pub(super) fn take(&self) -> Option<TransportWrapper> {
    let mut lock = self.buffer.lock();
    lock.pop_front()
  }

  pub(super) fn clone_buffer(&self) -> Vec<TransportWrapper> {
    let lock = self.buffer.lock();
    lock.iter().cloned().collect()
  }
}
