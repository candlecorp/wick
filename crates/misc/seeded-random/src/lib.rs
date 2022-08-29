#![doc = include_str!("../README.md")]
// !!START_LINTS
// Wasmflow lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![allow(unknown_lints)]
#![deny(
  clippy::expect_used,
  clippy::explicit_deref_methods,
  clippy::option_if_let_else,
  clippy::await_holding_lock,
  clippy::cloned_instead_of_copied,
  clippy::explicit_into_iter_loop,
  clippy::flat_map_option,
  clippy::fn_params_excessive_bools,
  clippy::implicit_clone,
  clippy::inefficient_to_string,
  clippy::large_types_passed_by_value,
  clippy::manual_ok_or,
  clippy::map_flatten,
  clippy::map_unwrap_or,
  clippy::must_use_candidate,
  clippy::needless_for_each,
  clippy::needless_pass_by_value,
  clippy::option_option,
  clippy::redundant_else,
  clippy::semicolon_if_nothing_returned,
  clippy::too_many_lines,
  clippy::trivially_copy_pass_by_ref,
  clippy::unnested_or_patterns,
  clippy::future_not_send,
  clippy::useless_let_if_seq,
  clippy::str_to_string,
  clippy::inherent_to_string,
  clippy::let_and_return,
  clippy::string_to_string,
  clippy::try_err,
  clippy::unused_async,
  clippy::missing_enforced_import_renames,
  clippy::nonstandard_macro_braces,
  clippy::rc_mutex,
  clippy::unwrap_or_else_default,
  clippy::manual_split_once,
  clippy::derivable_impls,
  clippy::needless_option_as_deref,
  clippy::iter_not_returning_iterator,
  clippy::same_name_method,
  clippy::manual_assert,
  clippy::non_send_fields_in_send_ty,
  clippy::equatable_if_let,
  bad_style,
  clashing_extern_declarations,
  const_err,
  dead_code,
  deprecated,
  explicit_outlives_requirements,
  improper_ctypes,
  invalid_value,
  missing_copy_implementations,
  missing_debug_implementations,
  mutable_transmutes,
  no_mangle_generic_items,
  non_shorthand_field_patterns,
  overflowing_literals,
  path_statements,
  patterns_in_fns_without_body,
  private_in_public,
  trivial_bounds,
  trivial_casts,
  trivial_numeric_casts,
  type_alias_bounds,
  unconditional_recursion,
  unreachable_pub,
  unsafe_code,
  unstable_features,
  unused,
  unused_allocation,
  unused_comparisons,
  unused_import_braces,
  unused_parens,
  unused_qualifications,
  while_true,
  missing_docs
)]
#![allow(unused_attributes, clippy::derive_partial_eq_without_eq)]
// !!END_LINTS
// Add exceptions here
#![allow(clippy::must_use_candidate)]

use std::sync::Arc;

use parking_lot::RwLock;
use rand::distributions::{Alphanumeric, Standard};
use rand::prelude::Distribution;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use uuid::Uuid;

#[allow(missing_copy_implementations)]
#[derive(Debug)]
/// This encapsulates the RNG seed into a separate, *uncopyable*, and *uncloneable* value so
/// it can not be accidentally propagated to another RNG without understanding the implication
/// of reusing seeds.
pub struct Seed(u64);

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
  pub fn rng(self) -> Random {
    Random::from_seed(self)
  }
}

impl std::fmt::Display for Seed {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug)]
#[must_use]
/// The main RNG data structure.
pub struct Random {
  rng: Arc<RwLock<ChaCha12Rng>>,
}

#[must_use]
pub(crate) fn new_seed() -> Seed {
  let mut rng = rand::thread_rng();
  Seed(rng.gen())
}

impl Random {
  /// Create a new RNG from a new, random seed.
  pub fn new() -> Self {
    Self::from_seed(new_seed())
  }

  /// Generated a new seed from this RNG.
  pub fn seed(&self) -> Seed {
    Seed(self.gen())
  }

  #[must_use]
  /// Function that delegates to [rand::rng::Rng::gen()]
  pub fn gen<T>(&self) -> T
  where
    Standard: Distribution<T>,
  {
    let mut rng = self.rng.write();
    rng.gen()
  }

  // Need to allow this lint so we can write an API that consumes the Seed even
  // though it's not technically necessary.
  #[allow(clippy::needless_pass_by_value)]
  /// Create a new [Random] RNG from a seed.
  pub fn from_seed(seed: Seed) -> Self {
    let rng = ChaCha12Rng::seed_from_u64(seed.0);
    Self {
      rng: Arc::new(RwLock::new(rng)),
    }
  }

  /// Utility function to generate a new [u32]
  pub fn u32(&self) -> u32 {
    self.gen()
  }

  /// Utility function to generate a new [i32]
  pub fn i32(&self) -> i32 {
    self.gen()
  }

  /// Utility function to generate a new [Vec<u8>]
  pub fn bytes(&self, length: usize) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::with_capacity(length);
    let mut rng = self.rng.write();
    for _ in 0..length {
      bytes.push(rng.gen());
    }
    bytes
  }

  /// Utility function to generate a new [String]
  pub fn string(&self, length: usize) -> String {
    let mut string: String = String::with_capacity(length);
    let mut rng = self.rng.write();

    for _ in 0..length {
      string.push(rng.gen());
    }
    string
  }

  /// Utility function to generate a new [String] consisting only of numbers and letters.
  pub fn alphanumeric(&self, length: usize) -> String {
    let mut rng = self.rng.write();
    let chars: String = std::iter::repeat(())
      .map(|()| rng.sample(Alphanumeric))
      .map(char::from)
      .take(length)
      .collect();
    chars
  }

  /// Utility function to generate a new [uuid::Uuid]
  pub fn uuid(&self) -> Uuid {
    let mut raw_bytes: [u8; 16] = [0; 16];
    let mut rng = self.rng.write();
    rng.fill(&mut raw_bytes);
    let bytes: uuid::Bytes = raw_bytes;
    let builder = uuid::Builder::from_bytes(bytes);
    builder.into_uuid()
  }

  /// Utility function that delegates to [rand::rng::Rng::gen_range()]
  pub fn range(&self, min: u32, max: u32) -> u32 {
    let mut rng = self.rng.write();
    rng.gen_range(min..max)
  }
}

impl Default for Random {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn bytes() {
    let rng = Random::from_seed(Seed(100000));
    let bytes1 = rng.bytes(10);
    let bytes2 = rng.bytes(10);
    assert_ne!(bytes1, bytes2);
    let rng = Random::from_seed(Seed(100000));
    let bytes2 = rng.bytes(10);
    assert_eq!(bytes1, bytes2);
  }
  #[test]
  fn string() {
    let rng = Random::from_seed(Seed(100000));
    let v1 = rng.string(10);
    let v2 = rng.string(10);
    assert_ne!(v1, v2);
    let rng = Random::from_seed(Seed(100000));
    let v2 = rng.string(10);
    assert_eq!(v1, v2);
  }

  #[test]
  fn alphanum() {
    let rng = Random::from_seed(Seed(100000));
    let v1 = rng.alphanumeric(10);
    let v2 = rng.alphanumeric(10);
    assert_ne!(v1, v2);
    let rng = Random::from_seed(Seed(100000));
    let v2 = rng.alphanumeric(10);
    assert_eq!(v1, v2);
  }

  #[test]
  fn uuid() {
    let rng = Random::from_seed(Seed(100000));
    let v1 = rng.uuid();
    let v2 = rng.uuid();
    assert_ne!(v1, v2);
    let rng = Random::from_seed(Seed(100000));
    let v2 = rng.uuid();
    assert_eq!(v1, v2);
  }
}
