// !!START_LINTS
// Wasmflow lints
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
#![allow(missing_docs)]

pub mod error;

pub(crate) type Result<T> = std::result::Result<T, error::LoadError>;
pub type Error = error::LoadError;

#[macro_use]
extern crate tracing;

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub async fn get_bytes_from_file(path: &Path) -> Result<Vec<u8>> {
  Ok(tokio::fs::read(path).await?)
}

pub async fn get_bytes_from_oci(path: &str, allow_latest: bool, allowed_insecure: &[String]) -> Result<Vec<u8>> {
  Ok(wasmflow_oci::fetch_oci_bytes(path, allow_latest, allowed_insecure).await?)
}

pub async fn get_bytes(location: &str, allow_latest: bool, allowed_insecure: &[String]) -> Result<Vec<u8>> {
  let path = Path::new(&location);
  if path.exists() {
    debug!(location, "load as file");
    Ok(get_bytes_from_file(path).await?)
  } else {
    let cache_path = cache_location("ocicache", location);
    if cache_path.exists() {
      debug!(
        path = %cache_path.to_string_lossy(),
        "load from cache"
      );

      let mut buf = vec![];
      let mut f = std::fs::File::open(cache_path)?;
      f.read_to_end(&mut buf)?;
      Ok(buf)
    } else {
      debug!(location, "load as OCI");
      let bytes = get_bytes_from_oci(location, allow_latest, allowed_insecure).await?;
      let mut f = std::fs::File::create(cache_path)?;
      f.write_all(&bytes)?;
      f.flush()?;
      Ok(bytes)
    }
  }
}

pub const CACHE_ROOT: &str = "wafl";
pub const CACHE_EXT: &str = "store";

#[must_use]
pub fn cache_location(bucket: &str, reference: &str) -> PathBuf {
  let path = std::env::temp_dir();
  let path = path.join(CACHE_ROOT);
  let path = path.join(bucket);
  let _ = ::std::fs::create_dir_all(&path);
  let reference = reference.replace(':', "_").replace('/', "_").replace('.', "_");
  let mut path = path.join(reference);
  path.set_extension(CACHE_EXT);

  path
}
