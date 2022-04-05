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
#![allow(unused_attributes)]
// !!END_LINTS
// Add exceptions here
#![allow(missing_docs)] // TODO

#[macro_use]
extern crate tracing;

mod dispatch;
pub mod error;
mod network;
pub use network::{Network, NetworkBuilder};
pub use providers::network_provider::Provider as NetworkProvider;
mod json_writer;
mod network_service;
mod providers;
pub mod utils;

pub mod prelude {
  pub use tokio_stream::StreamExt;
  pub use vino_codec::messagepack::{deserialize as mp_deserialize, serialize as mp_serialize};
  pub use vino_manifest::NetworkDefinition;
  pub use vino_packet::{packet, Packet};
  pub use vino_transport::{MessageTransport, TransportStream, TransportWrapper};

  pub use crate::dispatch::{DispatchError, InvocationResponse};
  pub use crate::network::Network;
  pub use crate::providers::network_provider::Provider as NetworkProvider;
  pub use crate::{SCHEMATIC_INPUT, SCHEMATIC_OUTPUT, SELF_NAMESPACE};
}

pub(crate) mod dev;

#[cfg(test)]
pub(crate) mod test;

pub type Error = error::RuntimeError;
pub use network_service::error::NetworkError;
pub use providers::error::ProviderError;

/// The reserved reference name for schematic input. Used in schematic manifests to denote schematic input.
pub const SCHEMATIC_INPUT: &str = "<input>";

/// The reserved reference name for schematic output. Used in schematic manifests to denote schematic output.
pub const SCHEMATIC_OUTPUT: &str = "<output>";

/// The reserved namespace for references to internal schematics.
pub const SELF_NAMESPACE: &str = "self";

/// The reserved namespace for Vino's initial native API.
pub const VINO_V0_NAMESPACE: &str = "vino";

pub const CORE_PORT_SEED: &str = "seed";
