//! Wick RPC SDK

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

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::{make_rpc_client, RpcClient};

/// Error module.
pub mod error;

mod generated;

/// Utility and conversion types.
pub mod types;

/// Module with generated Tonic & Protobuf code.
pub use generated::wick as rpc;
use tokio_stream::StreamExt;
pub use types::*;

/// The crate's error type.
pub type Error = crate::error::RpcError;

pub fn convert_tonic_streaming(mut streaming: tonic::Streaming<rpc::Packet>) -> wick_packet::PacketStream {
  let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
  tokio::spawn(async move {
    while let Some(packet) = streaming.next().await {
      let result: Result<wick_packet::Packet, wick_packet::Error> = match packet {
        Ok(o) => Ok(o.into()),
        Err(e) => Err(wick_packet::Error::Component(e.to_string())),
      };
      let _ = tx.send(result);
    }
  });

  wick_packet::PacketStream::new(Box::new(tokio_stream::wrappers::UnboundedReceiverStream::new(rx)))
}

#[macro_export]
macro_rules! dispatch {
  ($inv:expr, {$($name:expr => $handler:path),*,}) => {
    {
      match $inv.target.operation_id() {
        $($name => $handler($inv).await?,)*
        _ => {
          unreachable!()
        }
      }
  }};
}
