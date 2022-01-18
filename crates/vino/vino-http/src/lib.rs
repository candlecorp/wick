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
  // next version, see: https://github.com/rust-lang/rust-clippy/blob/master/CHANGELOG.md
  // clippy::manual_split_once,
  // clippy::derivable_impls,
  // clippy::needless_option_as_deref,
  // clippy::iter_not_returning_iterator,
  // clippy::same_name_method,
  // clippy::manual_assert,
  // clippy::non_send_fields_in_send_ty,
  // clippy::equatable_if_let,
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
// !!END_LINTS
// Add exceptions here
#![allow(missing_docs)]

pub use config::Config;
use vino_rpc::SharedRpcHandler;

mod config;
mod cors;
mod error;
pub mod service;

use std::future::Future;
use std::pin::Pin;

pub use error::HttpError as Error;

use crate::service::ProviderService;

#[macro_use]
extern crate tracing;

/// enable a vino provider to handle http web requests with the default configuration.
///
/// Shortcut for `vino_http::config().enable(service)`
pub fn enable(provider: SharedRpcHandler) -> ProviderService {
  config().enable(provider)
}

/// returns a default [`Config`] instance for configuring services.
///
/// ## Example
///
/// ```
/// let config = vino_http::config()
///      .allow_origins(vec!["http://foo.com"])
///      .allow_credentials(false)
///      .expose_headers(vec!["x-request-id"]);
///
/// // let greeter = config.enable(Greeter);
/// ```
pub fn config() -> Config {
  Config::default()
}

// type BoxError = Box<dyn std::error::Error + Send + Sync>;
type BoxFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send>>;
