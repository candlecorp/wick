//! Cross-platform normalization of directories and other configuration related to Wick.

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
#![allow()]

mod directories;
mod directory;
mod error;
mod file;

use std::path::PathBuf;

pub use directories::Directories;
pub use error::Error;

#[derive(Debug, Clone, getset::Getters)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// A one-stop shop for all the environment-specific configuration for the Wick project.
pub struct Settings {
  /// Local directories for wick to store and retrieve files.
  #[getset(get = "pub")]
  local: Directories,
  /// Global directories for wick to store and retrieve files.
  #[getset(get = "pub")]
  global: Directories,
  /// The global state directory wick uses to store data while running.
  #[cfg_attr(feature = "serde", serde(serialize_with = "display_path"))]
  #[getset(get = "pub")]
  data: PathBuf,
  /// The location to look for user configuration.
  #[cfg_attr(feature = "serde", serde(serialize_with = "display_path"))]
  #[getset(get = "pub")]
  config_dir: PathBuf,
  /// The basename of the user configuration file.
  #[getset(get = "pub")]
  configfile_basename: String,
}
#[cfg(feature = "serde")]
fn display_path<S>(value: impl AsRef<std::path::Path>, serializer: S) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer,
{
  serializer.collect_str(value.as_ref().to_string_lossy().as_ref())
}

impl Default for Settings {
  fn default() -> Self {
    Self::new()
  }
}

impl Settings {
  #[must_use]
  ///
  pub fn new() -> Self {
    Self {
      local: Directories::new(&directory::relative_root()),
      global: Directories::new(&directory::global_root()),
      data: directory::global_data_dir(),
      config_dir: directory::user_config_dir(),
      configfile_basename: file::CONFIG_FILE_NAME.to_owned(),
    }
  }

  #[must_use]
  /// Returns the local directories if condition is true, otherwise returns the global directories.
  pub fn local_if(&self, condition: bool) -> &Directories {
    if condition {
      &self.local
    } else {
      &self.global
    }
  }
}
