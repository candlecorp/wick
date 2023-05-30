#[allow(missing_copy_implementations)]
#[derive(Debug)]
/// This encapsulates the RNG seed into a separate, *uncopyable*, and *uncloneable* value so
/// it can not be accidentally propagated to another RNG without understanding the implication
/// of reusing seeds.
pub struct Seed(pub(crate) u64);

impl Seed {
  /// If you absolutely need to create a new seed from a raw value, use this function.
  /// It's "unsafe" not because of memory reasons but because blindly reusing seed values
  /// can get you into tough-to-troubleshoot situations.
  ///
  /// It's better to generate new seeds and new RNGs from those seeds.
  pub fn unsafe_new(seed: u64) -> Self {
    Self(seed)
  }

  /// Creates a new RNG from this seed.
  #[cfg(feature = "rng")]
  pub fn rng(self) -> crate::Random {
    crate::Random::from_seed(self)
  }
}

impl std::fmt::Display for Seed {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}
