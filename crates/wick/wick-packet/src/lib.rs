#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/104781277?s=96&v=4")]
#![doc = include_str!("../README.md")]
// !!START_LINTS
// Wick lints
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
  clippy::if_then_some_else_none,
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
#![allow(unused_attributes)]
// !!END_LINTS
// Add exceptions here
#![allow(missing_docs)]

mod b64_bytes;
mod collection_link;
mod entity;
mod error;
mod inherent;
#[cfg(feature = "invocation")]
mod invocation;
mod macros;
mod metadata;
mod output;
mod packet;
mod packet_stream;
mod stream_map;
mod wrapped_type;

pub use collection_link::ComponentReference;
pub use entity::Entity;
pub use error::{Error, ParseError};
pub use inherent::InherentData;
#[cfg(feature = "invocation")]
pub use invocation::Invocation;
pub use metadata::{Flags, WickMetadata, CLOSE_BRACKET, DONE_FLAG, OPEN_BRACKET};
pub use output::Output;
pub use packet::{from_raw_wasmrs, from_wasmrs, into_wasmrs, Packet, PacketError, PacketPayload};
pub use packet_stream::{PacketSender, PacketStream};
pub use stream_map::StreamMap;
pub use wasmrs::Metadata;
pub use wasmrs_rx;
pub use wasmrs_rx::{Flux, FluxChannel, FluxReceiver, Mono, Observable, Observer};
pub use wrapped_type::TypeWrapper;

#[cfg(feature = "rt-tokio")]
mod runtime;
pub use b64_bytes::Base64Bytes;
#[cfg(feature = "rt-tokio")]
pub use runtime::split_stream;
