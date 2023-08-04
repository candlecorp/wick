use std::collections::VecDeque;
use std::fmt::Debug;
use std::ops::RangeBounds;

use parking_lot::Mutex;
use wick_packet::Packet;

type PacketType = Packet;

pub(super) struct PortBuffer {
  buffer: Mutex<VecDeque<PacketType>>,
}

impl Debug for PortBuffer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let len = self.buffer.lock().len();
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
    self.buffer.lock().push_back(value);
  }

  pub(super) fn is_empty(&self) -> bool {
    self.buffer.lock().is_empty()
  }

  // pub(super) fn len(&self) -> usize {
  //   self.buffer.lock().len()
  // }

  pub(super) fn take(&self) -> Option<PacketType> {
    self.buffer.lock().pop_front()
  }

  pub(super) fn drain<R>(&self, range: R) -> Vec<PacketType>
  where
    R: RangeBounds<usize>,
  {
    self.buffer.lock().drain(range).collect()
  }

  // pub(super) fn clone_buffer(&self) -> Vec<PacketType> {
  //   self.buffer.lock().iter().cloned().collect()
  // }
}
