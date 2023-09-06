#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/104781277?s=96&v=4")]
#![doc = include_str!("../README.md")]
// !!START_LINTS
// Wick lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![allow(unknown_lints)]
#![deny(
  clippy::await_holding_lock,
  clippy::borrow_as_ptr,
  clippy::branches_sharing_code,
  clippy::cast_lossless,
  clippy::clippy::collection_is_never_read,
  clippy::cloned_instead_of_copied,
  clippy::cognitive_complexity,
  clippy::create_dir,
  clippy::deref_by_slicing,
  clippy::derivable_impls,
  clippy::derive_partial_eq_without_eq,
  clippy::equatable_if_let,
  clippy::exhaustive_structs,
  clippy::expect_used,
  clippy::expl_impl_clone_on_copy,
  clippy::explicit_deref_methods,
  clippy::explicit_into_iter_loop,
  clippy::explicit_iter_loop,
  clippy::filetype_is_file,
  clippy::flat_map_option,
  clippy::format_push_string,
  clippy::fn_params_excessive_bools,
  clippy::future_not_send,
  clippy::get_unwrap,
  clippy::implicit_clone,
  clippy::if_then_some_else_none,
  clippy::impl_trait_in_params,
  clippy::implicit_clone,
  clippy::inefficient_to_string,
  clippy::inherent_to_string,
  clippy::iter_not_returning_iterator,
  clippy::large_types_passed_by_value,
  clippy::large_include_file,
  clippy::let_and_return,
  clippy::manual_assert,
  clippy::manual_ok_or,
  clippy::manual_split_once,
  clippy::manual_let_else,
  clippy::manual_string_new,
  clippy::map_flatten,
  clippy::map_unwrap_or,
  clippy::missing_enforced_import_renames,
  clippy::missing_assert_message,
  clippy::missing_const_for_fn,
  clippy::must_use_candidate,
  clippy::mut_mut,
  clippy::needless_for_each,
  clippy::needless_option_as_deref,
  clippy::needless_pass_by_value,
  clippy::needless_collect,
  clippy::needless_continue,
  clippy::non_send_fields_in_send_ty,
  clippy::nonstandard_macro_braces,
  clippy::option_if_let_else,
  clippy::option_option,
  clippy::rc_mutex,
  clippy::redundant_else,
  clippy::same_name_method,
  clippy::semicolon_if_nothing_returned,
  clippy::str_to_string,
  clippy::string_to_string,
  clippy::too_many_lines,
  clippy::trivially_copy_pass_by_ref,
  clippy::trivial_regex,
  clippy::try_err,
  clippy::unnested_or_patterns,
  clippy::unused_async,
  clippy::unwrap_or_else_default,
  clippy::useless_let_if_seq,
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
#![warn(clippy::exhaustive_enums)]
#![allow(unused_attributes, clippy::derive_partial_eq_without_eq, clippy::box_default)]
// !!END_LINTS
// Add exceptions here
#![allow()]

/// A module implementing custom serializers/deserializers for wick types.
pub mod serde_util;

//
//
/// Macros used by wick components and generated code.
pub mod macros;
//
//
/// Re-export of the complete [bytes] module.
#[cfg(feature = "bytes")]
pub use bytes;
//
//
/// Re-exported methods used for `datetime` types.
#[cfg(feature = "datetime")]
pub mod datetime {
  /// Re-export of the chrono crate.
  pub use chrono;
  pub use chrono::format::strftime;
  pub use chrono::offset::TimeZone;
  pub use chrono::serde::ts_milliseconds;
  pub use chrono::{DateTime as ChronoDateTime, Utc};
  pub use wick_packet::{date_from_millis, serde, DateTime};
}
//
//
/// Re-export of serde_json utilities;
#[cfg(feature = "json")]
pub use serde_json::{from_slice, from_str, from_value, json, to_value, Map, Value};
//
//
/// Re-export of tokio_stream utilities;
pub use tokio_stream::{empty, iter as iter_raw, once as once_raw, Stream, StreamExt};
//
//
/// Re-export of wasmrs_guest.
#[cfg(target_family = "wasm")]
pub use wasmrs_guest;
//
//
/// Re-export of wasmrs_rx traits and core types.
pub use wasmrs_rx::{Flux, FluxChannel, Observable, Observer};
//
//
/// Old export name for `wick_packet`
#[cfg(deprecated)]
pub use wick_packet as packet;
//
//
/// Re-export of [wick_packet::Base64Bytes] as [Bytes].
pub use wick_packet::{Base64Bytes as Bytes, Packet, PacketSender, Port, ValuePort};
//
//
/// Other re-exported crates;
pub use {flow_component, paste, wasmrs, wasmrs_codec, wasmrs_runtime as runtime, wasmrs_rx, wick_packet};

//
//
/// Generic boxed-error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

//
//
/// A stream of `Result<T,BoxError>`.
pub type WickStream<T> = wasmrs_rx::BoxFlux<T, BoxError>;

/// Create a stream of `Result<T, BoxError>` that yields one value and ends.
///
/// # Example
///
/// ```
/// # use wick_component::prelude::*;
///
/// let mut stream = once(42);
/// assert_eq!(stream.next().await, Some(Ok(42)));
/// assert_eq!(stream.next().await, None);
/// ```
///
pub fn once<T>(value: T) -> impl Stream<Item = Result<T, BoxError>> {
  tokio_stream::once(Ok(value))
}

/// Create a stream of `Result<T, BoxError>` from an iterator of type T.
///
/// This is a convenience function for creating a Result/TryStream from an iterator of `Ok` values.
///
/// # Example
///
/// ```
/// # use wick_component::prelude::*;
///
/// let mut stream = iter(vec![1, 2, 3]);
/// assert_eq!(stream.next().await, Some(Ok(1)));
/// assert_eq!(stream.next().await, Some(Ok(2)));
/// assert_eq!(stream.next().await, Some(Ok(3)));
/// assert_eq!(stream.next().await, None);
/// ```
///
pub fn iter<I, O>(i: I) -> impl Stream<Item = Result<O, BoxError>>
where
  I: IntoIterator<Item = O>,
{
  tokio_stream::iter(i.into_iter().map(|i| Ok(i)))
}

mod adapters;
/// Functions and macros for common operation types.
pub use adapters::*;

mod outputs;
pub use outputs::{Broadcast, SingleOutput};
//
//
/// The proc macro to automatically implement common operaton types.
pub use wick_operation::operation;

/// Useful userland utilities that can be exported via `use wick_component::prelude::*`
#[cfg(deprecated = "Use `wick_component::*` instead")]
#[doc(hidden)]
pub mod prelude {
  pub use super::*;
}
