//! Wick Manifest implementation

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
#![allow(
  unused_attributes,
  clippy::derive_partial_eq_without_eq,
  clippy::box_default,
  clippy::exhaustive_enums
)]
// !!END_LINTS
// Add exceptions here
#![allow()]

/// Structures and functions for auditing Wick Manifests.
#[cfg(feature = "config")]
pub mod audit;
mod default;
mod helpers;
mod traits;

/// Data structures, traits, and errors associated with Lockdown configuration.
#[cfg(feature = "config")]
pub mod lockdown;

/// The main configuration module.
#[cfg(feature = "config")]
pub mod config;
/// Wick Manifest error.
pub mod error;
mod utils;

#[allow(unused)]
pub(crate) use utils::impl_from_for;

/// Wick Manifest v0 implementation.
#[cfg(feature = "v0")]
pub mod v0;

/// Wick Manifest v1 implementation.
#[cfg(feature = "v1")]
pub mod v1;

/// Methods to read and load raw configurations.
pub mod load;

pub use default::{parse_default, process_default, ERROR_STR};

/// The crate's error type.
pub type Error = crate::error::ManifestError;

#[cfg(feature = "config")]
mod feature_config {
  pub use crate::config::WickConfiguration;
  use crate::error::ManifestError;
  /// The type associated with a resolver function.
  pub type Resolver = dyn Fn(&str) -> Result<crate::config::OwnedConfigurationItem, ManifestError> + Send + Sync;
  pub use crate::traits::*;

  // Todo: flesh out per-component validation of configuration.
  #[doc(hidden)]
  pub trait ConfigValidation {
    type Config;
    fn validate(config: &Self::Config, resolver: &Resolver) -> Result<(), flow_component::ComponentError>;
  }
  pub use wick_asset_reference::{
    normalize_path,
    AssetReference,
    Error as AssetError,
    FetchOptions,
    FetchableAssetReference,
  };
}
#[cfg(feature = "config")]
pub use feature_config::*;

pub(crate) type Result<T> = std::result::Result<T, Error>;
