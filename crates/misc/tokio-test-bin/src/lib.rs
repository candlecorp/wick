// !!START_LINTS
// Wasmflow lints
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
#![allow(unused_attributes, clippy::derive_partial_eq_without_eq)]
// !!END_LINTS
// Add exceptions here
#![allow(clippy::expect_used)]

//! A module for getting the crate binary in an integration test.
//!
//! If you are writing a command-line interface app then it is useful to write
//! an integration test that uses the binary. You most likely want to launch the
//! binary and inspect the output. This module lets you get the binary so it can
//! be tested.
//!
//! # Examples
//!
//! basic usage:
//!
//! ```no_run
//! # async fn run() -> Result<(), std::io::Error> {
//!   let output = tokio_test_bin::get_test_bin("my_cli_app")
//!     .output().await?;
//!
//!   assert_eq!(
//!     String::from_utf8_lossy(&output.stdout),
//!     "Output from my CLI app!\n"
//!   );
//! # Ok(())
//! # }
//! ```
//!
//! Refer to the [`std::process::Command` documentation](https://doc.rust-lang.org/std/process/struct.Command.html)
//! for how to pass arguments, check exit status and more.

/// This crate's error module.
pub mod error;
pub use error::Error;

/// Returns the crate's binary as a `Command` that can be used for integration
/// tests.
///
/// # Arguments
///
/// * `bin_name` - The name of the binary you want to test.
///
/// # Remarks
///
/// It panics on error. This is by design so the test that uses it fails.
#[must_use]
pub fn get_test_bin(bin_name: &str) -> tokio::process::Command {
  // Create full path to binary
  let mut path = get_test_bin_dir();
  path.push(bin_name);
  path.set_extension(std::env::consts::EXE_EXTENSION);

  assert!(path.exists());

  // Create command
  tokio::process::Command::new(path.into_os_string())
}

/// Returns the directory of the crate's binary.
///
/// # Remarks
///
/// It panics on error. This is by design so the test that uses it fails.
#[must_use]
fn get_test_bin_dir() -> std::path::PathBuf {
  // Cargo puts the integration test binary in target/debug/deps
  let current_exe = std::env::current_exe().expect("Failed to get the path of the integration test binary");

  let current_dir = current_exe
    .parent()
    .expect("Failed to get the directory of the integration test binary");

  assert!(current_dir.ends_with("target/debug/deps") || current_dir.ends_with("target/release/deps"));

  let test_bin_dir = current_dir.parent().expect("Failed to get the binary folder");
  test_bin_dir.to_owned()
}
