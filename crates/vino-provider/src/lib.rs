//! Vino provider

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
  clippy::if_then_some_else_none,
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
#![allow()]

use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};

use async_trait::async_trait;
use vino_rpc::port::PortStream;
/// The crate's error module
pub mod error;

/// The crate's error type
pub type Error = error::ProviderError;

/// The type of a provider's context.
pub type Context<T> = Arc<Mutex<T>>;

#[async_trait]
/// Trait used by auto-generated provider components. You shouldn't need to implement this if you are using Vino's code generator.
pub trait VinoProviderComponent {
  /// The provider context to pass to the component.
  type Context;
  /// To return the name of the component.
  fn get_name(&self) -> String;
  /// To return the input ports and type signatures.
  fn get_input_ports(&self) -> Vec<(&'static str, &'static str)>;
  /// To return the output ports and type signatures.
  fn get_output_ports(&self) -> Vec<(&'static str, &'static str)>;
  /// The wrapper method that is called to execute the component's job.
  async fn job_wrapper(
    &self,
    context: Arc<Mutex<Self::Context>>,
    data: HashMap<String, Vec<u8>>,
  ) -> Result<PortStream, Box<ProviderComponentError>>;
}

pub use vino_entity as entity;
pub use vino_rpc::ComponentSignature;

use self::error::ProviderComponentError;
