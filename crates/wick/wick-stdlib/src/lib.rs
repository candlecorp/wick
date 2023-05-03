// !!START_LINTS
// Wick lints
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
#![allow(unused_attributes, clippy::derive_partial_eq_without_eq, clippy::box_default)]
// !!END_LINTS
// Add exceptions here
#![allow(missing_docs)]

mod macros;
mod operations;

use flow_component::{Component, RuntimeCallback};
use seeded_random::Seed;
use wick_interface_types::{component, ComponentSignature};
use wick_packet::{Invocation, PacketStream};
use wick_rpc::{dispatch, RpcHandler};

#[macro_use]
extern crate tracing;

#[derive(Clone, Debug, Copy)]
pub struct Context {}

#[derive(Debug)]
#[must_use]
pub struct Collection {
  #[allow(unused)]
  seed: Seed,
  signature: ComponentSignature,
}

impl Collection {
  pub fn new(seed: Seed) -> Self {
    let sig = component! {
        name: "wick-stdlib",
        version: "0.1.0",
        operations: {
          "core::error" => {
            inputs: { "input" => "string" },
            outputs: { "output" => "string" },
          },
          "core::log" => {
            inputs: { "input" => "string" },
            outputs: { "output" => "string" },
          },
          "core::panic" => {
            inputs: { "input" => "string" },
            outputs: { "output" => "string" },
          },
          "math::add" => {
            inputs: { "left" => "u64", "right" => "u64" },
            outputs: { "output" => "u64" },
          },
          "math::subtract" => {
            inputs: { "left" => "u64", "right" => "u64" },
            outputs: { "output" => "u64" },
          },
          "rand::bytes" => {
            inputs: { "seed" => "u64", "length" => "u32" },
            outputs: { "output" => "bytes" },
          },
          "rand::string" => {
            inputs: { "seed" => "u64", "length" => "u32" },
            outputs: { "output" => "string" },
          },
          "rand::uuid" => {
            inputs: { "seed" => "u64" },
            outputs: { "output" => "string" },
          },
          "string::concat" => {
            inputs: { "left" => "string", "right" => "string" },
            outputs: { "output" => "string" },
          }
        }
    };
    Self { seed, signature: sig }
  }
}

impl Component for Collection {
  fn handle(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    _data: Option<wick_packet::OperationConfig>,
    _callback: std::sync::Arc<RuntimeCallback>,
  ) -> flow_component::BoxFuture<Result<PacketStream, flow_component::ComponentError>> {
    let target = invocation.target_url();
    trace!("stdlib invoke: {}", target);

    Box::pin(async move {
      let stream = dispatch!(invocation, stream, {
            "core::error" => operations::core::error::job,
            "core::log" => operations::core::log::job,
            "core::panic" => operations::core::panic::job,
            "math::add" => operations::math::add::job,
            "math::subtract" => operations::math::subtract::job,
            "rand::bytes" => operations::rand::bytes::job,
            "rand::string" => operations::rand::string::job,
            "rand::uuid" => operations::rand::uuid::job,
            "string::concat" => operations::string::concat::job,
      });
      Ok(stream)
    })
  }

  fn list(&self) -> &ComponentSignature {
    &self.signature
  }
}

impl RpcHandler for Collection {}
