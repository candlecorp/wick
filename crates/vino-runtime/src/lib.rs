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

#[macro_use]
mod macros {
  /// log_ie!(Result, u16) takes a result and an error number, maps the error to an InternalError and logs it.
  macro_rules! log_ie {
    ($expr:expr, $errnum: literal $(,)?) => {{
      let result = $expr;
      let ie = InternalError($errnum);
      if result.is_err() {
        error!("{}", ie);
      }
      result.map_err(|_| ie)
    }};
  }

  macro_rules! actix_try {
    ($expr:expr $(,)?) => {
      match $expr {
        Ok(val) => val,
        Err(err) => {
          error!("Unexpected error: {}", err);
          return ActorResult::reply(Err(From::from(err)));
        }
      }
    };
  }

  macro_rules! actix_ensure_ok {
    ($expr:expr $(,)?) => {
      match $expr {
        Ok(val) => val,
        Err(err) => {
          return ActorResult::reply(err);
        }
      }
    };
  }
}

#[macro_use]
extern crate derivative;

#[macro_use]
extern crate vino_macros;

#[macro_use]
extern crate tracing;

mod dispatch;
pub mod error;
pub mod models;
pub mod network;
mod network_service;
mod providers;
mod schematic_service;
mod transaction;
pub mod utils;

pub mod prelude {
  pub use tokio_stream::StreamExt;
  pub use vino_codec::messagepack::{
    deserialize as mp_deserialize,
    serialize as mp_serialize,
  };
  pub use vino_component::{
    packet,
    Packet,
  };
  pub use vino_manifest::NetworkDefinition;
  pub use vino_transport::MessageTransport;

  pub use crate::dispatch::{
    Invocation,
    InvocationResponse,
    ResponseStream,
  };
  pub use crate::network::Network;
  pub use crate::utils::helpers::*;
  pub use crate::{
    SCHEMATIC_INPUT,
    SCHEMATIC_OUTPUT,
    SELF_NAMESPACE,
  };
}

pub(crate) mod dev;
#[cfg(test)]
pub(crate) mod test;

pub type Result<T> = std::result::Result<T, error::VinoError>;
pub type Error = error::VinoError;

/// The reserved reference name for schematic input. Used in schematic manifests to denote schematic input.
pub const SCHEMATIC_INPUT: &str = "<input>";
/// The reserved reference name for schematic output. Used in schematic manifests to denote schematic output.
pub const SCHEMATIC_OUTPUT: &str = "<output>";
/// The reserved port name to use when sending an asynchronous error from a component.
pub const COMPONENT_ERROR: &str = "<error>";
/// The reserved namespace for references to internal schematics.
pub const SELF_NAMESPACE: &str = "self";
