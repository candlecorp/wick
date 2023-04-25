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

use std::sync::Arc;

use wick_interface_types::ComponentSignature;
use wick_packet::{ComponentReference, InherentData, Invocation, PacketStream, StreamMap};

pub type SharedComponent = Arc<dyn Component + Send + Sync>;

pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn futures::Future<Output = T> + Send + 'a>>;

pub use serde_json::Value;

pub type RuntimeCallback = dyn Fn(
    ComponentReference,
    String,
    PacketStream,
    Option<InherentData>,
  ) -> BoxFuture<'static, Result<PacketStream, ComponentError>>
  + Send
  + Sync;

#[must_use]
pub fn panic_callback() -> Arc<RuntimeCallback> {
  Arc::new(|_, _, _, _| {
    Box::pin(async move {
      panic!("Panic callback invoked. This should never happen outside of tests.");
    })
  })
}

#[derive(Debug)]
#[must_use]
pub struct ComponentError {
  source: Box<dyn std::error::Error + Send + Sync>,
}
impl std::error::Error for ComponentError {}
impl std::fmt::Display for ComponentError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Component error: {}", self.source)
  }
}
impl ComponentError {
  pub fn new(source: impl std::error::Error + Send + Sync + 'static) -> Self {
    Self {
      source: Box::new(source),
    }
  }

  pub fn message(msg: &str) -> Self {
    Self {
      source: Box::new(GenericError(msg.to_owned())),
    }
  }
}
impl From<Box<dyn std::error::Error + Send + Sync>> for ComponentError {
  fn from(source: Box<dyn std::error::Error + Send + Sync>) -> Self {
    Self { source }
  }
}

impl From<anyhow::Error> for ComponentError {
  fn from(source: anyhow::Error) -> Self {
    Self::message(&source.to_string())
  }
}

#[derive(Debug)]
struct GenericError(String);
impl std::error::Error for GenericError {}
impl std::fmt::Display for GenericError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

pub trait Component {
  fn handle(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    data: Option<Value>,
    callback: Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>>;
  fn list(&self) -> &ComponentSignature;
  fn init(&self) -> BoxFuture<Result<(), ComponentError>> {
    // Override if you need a more explicit init.
    Box::pin(async move { Ok(()) })
  }

  fn shutdown(&self) -> BoxFuture<Result<(), ComponentError>> {
    // Override if you need a more explicit shutdown.
    Box::pin(async move { Ok(()) })
  }
}

pub trait Operation {
  fn handle(&self, payload: StreamMap, data: Option<Value>) -> BoxFuture<Result<PacketStream, ComponentError>>;
}
