//! Flow expression parser

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

/// Module for parsing parts of a manifest.
pub mod parse;

/// Error module.
pub mod error;

use std::str::FromStr;

pub use parse::v0::parse_id;

use crate::error::ParserError;

/// The crate's error type.
pub type Error = ParserError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[must_use]
/// A node instance
pub enum InstanceTarget {
  /// A flow input node.
  Input,
  /// A flow output node.
  Output,
  /// A reserved namespace for built-in nodes.
  Core,
  /// An unspecified node.
  Default,
  #[doc(hidden)]
  Link,
  /// A named node instance.
  Named(String),
  /// An instance created inline.
  Path(String, String),
}

impl InstanceTarget {
  /// Returns [self] unless self is [InstanceTarget::Default], in which case it returns [other].
  pub fn or(self, other: InstanceTarget) -> InstanceTarget {
    match self {
      InstanceTarget::Default => other,
      _ => self,
    }
  }

  /// Get the id of the instance target.
  #[must_use]
  pub fn id(&self) -> &str {
    match self {
      InstanceTarget::Input => parse::SCHEMATIC_INPUT,
      InstanceTarget::Output => parse::SCHEMATIC_OUTPUT,
      InstanceTarget::Core => parse::CORE_ID,
      InstanceTarget::Default => panic!("Cannot get id of default instance"),
      InstanceTarget::Link => parse::NS_LINK,
      InstanceTarget::Named(name) => name,
      InstanceTarget::Path(_, id) => id,
    }
  }

  /// Create a new [InstanceTarget::Named] from a string.
  pub fn named(name: impl AsRef<str>) -> Self {
    Self::Named(name.as_ref().to_owned())
  }

  /// Create a new [InstanceTarget::Path] from a path and id.
  pub fn path(path: impl AsRef<str>, id: impl AsRef<str>) -> Self {
    Self::Path(path.as_ref().to_owned(), id.as_ref().to_owned())
  }
}

impl FromStr for InstanceTarget {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    parse::v1::parse_instance(s)
  }
}

impl std::fmt::Display for InstanceTarget {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      InstanceTarget::Input => f.write_str(parse::SCHEMATIC_INPUT),
      InstanceTarget::Output => f.write_str(parse::SCHEMATIC_OUTPUT),
      InstanceTarget::Core => f.write_str(parse::CORE_ID),
      InstanceTarget::Default => f.write_str("<>"),
      InstanceTarget::Link => f.write_str(parse::NS_LINK),
      InstanceTarget::Named(name) => f.write_str(name),
      InstanceTarget::Path(path, id) => write!(f, "{}[{}]", path, id),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[must_use]
/// A port on a node instance, used to connect node instances together.
pub struct ConnectionTarget {
  pub(crate) target: InstanceTarget,
  pub(crate) port: String,
}

impl ConnectionTarget {
  /// Create a new ConnectionTarget.
  pub fn new(target: InstanceTarget, port: impl AsRef<str>) -> Self {
    Self {
      target,
      port: port.as_ref().to_owned(),
    }
  }

  /// Get the target port
  #[must_use]
  pub fn port(&self) -> &str {
    &self.port
  }

  /// Get the target instance
  pub fn target(&self) -> &InstanceTarget {
    &self.target
  }
}

impl std::fmt::Display for ConnectionTarget {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}.{}", self.target, self.port)
  }
}
