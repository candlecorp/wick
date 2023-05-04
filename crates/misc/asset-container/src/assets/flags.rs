#![allow(clippy::same_name_method)]

bitflags::bitflags! {
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  pub struct AssetFlags: u32 {
      const Lazy = 0b00000001;
      const NoFetch = 0b00000010;
  }
}

impl PartialEq<u32> for AssetFlags {
  fn eq(&self, other: &u32) -> bool {
    self.bits() == *other
  }
}

impl PartialEq<AssetFlags> for u32 {
  fn eq(&self, other: &AssetFlags) -> bool {
    other.bits() == *self
  }
}
