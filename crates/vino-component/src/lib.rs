//! The Vino component library. Contains the versioned output structures and related functions.

// !!START_LINTS
// Vino lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
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
    // clippy::too_many_lines,
    clippy::trivially_copy_pass_by_ref,
    clippy::unnested_or_patterns,
    clippy::future_not_send,
    clippy::useless_let_if_seq,
    clippy::str_to_string,
    clippy::inherent_to_string,
    clippy::let_and_return,
    clippy::string_to_string,
    clippy::try_err,
    clippy::if_then_some_else_none,
    bad_style,
    clashing_extern_declarations,
    const_err,
    // dead_code,
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
    path_statements ,
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
    // unused,
    unused_allocation,
    unused_comparisons,
    unused_import_braces,
    unused_parens,
    unused_qualifications,
    while_true,
    // missing_docs
)]
// !!END_LINTS
// Add exceptions here
#![allow()]

/// Version 0 of the output format
pub mod v0;
/// Module for Packet, the abstraction over different versions
pub mod packet {
  pub use crate::v0;
}
/// The WasCap claims for a component module
pub mod claims;

use serde::{
  Deserialize,
  Serialize,
};
use vino_codec::messagepack::deserialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// The output payload that component's push out of output ports
pub enum Packet {
  /// Version 0 of the payload format (unstable)
  #[serde(rename = "0")]
  V0(v0::Payload),
}

/// The error type used when attempting to deserialize a [Packet]
#[derive(Debug)]
pub enum Error {
  /// Invalid payload
  Invalid,
  /// Packet was an Exception
  Exception(String),
  /// Packet was an Error
  Error(String),
  /// An internal error occurred
  InternalError(vino_codec::Error),
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::Invalid => write!(f, "Invalid"),
      Error::Exception(v) => write!(f, "{}", v),
      Error::Error(v) => write!(f, "{}", v),
      Error::InternalError(e) => write!(f, "{}", e.to_string()),
    }
  }
}

impl std::error::Error for Error {}

impl Packet {
  /// Try to deserialize a [Packet] into the target type
  pub fn try_into<'de, T: Deserialize<'de>>(self) -> Result<T, Error> {
    match self {
      Packet::V0(v) => match v {
        v0::Payload::Invalid => Err(Error::Invalid),
        v0::Payload::Exception(v) => Err(Error::Exception(v)),
        v0::Payload::Error(v) => Err(Error::Error(v)),
        v0::Payload::MessagePack(buf) => deserialize::<T>(&buf).map_err(Error::InternalError),
        v0::Payload::Close => todo!(),
        v0::Payload::OpenBracket => todo!(),
        v0::Payload::CloseBracket => todo!(),
      },
    }
  }
}

impl From<&Vec<u8>> for Packet {
  fn from(buf: &Vec<u8>) -> Self {
    match deserialize::<Packet>(buf) {
      Ok(packet) => packet,
      Err(e) => Packet::V0(v0::Payload::Error(format!(
        "Error deserializing packet: {}",
        e
      ))),
    }
  }
}

impl From<&[u8]> for Packet {
  fn from(buf: &[u8]) -> Self {
    match deserialize::<Packet>(buf) {
      Ok(packet) => packet,
      Err(e) => Packet::V0(v0::Payload::Error(format!(
        "Error deserializing packet: {}",
        e
      ))),
    }
  }
}
