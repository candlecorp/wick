//! OCI fetch and utility package

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

mod architecture_map;
/// This crate's error module.
pub mod error;
mod manifest;
mod options;
/// OCI utilities related to pushing and pulling Wick packages.
pub mod package;
mod pull;
mod push;
mod utils;

pub use architecture_map::{
  generate_archmap,
  ArchitectureEntry,
  ArchitectureMap,
  MultiArchManifest,
  MultiArchManifestEntry,
};
pub use error::OciError as Error;
pub use manifest::*;
pub use oci_distribution::client::ClientProtocol;
pub use options::*;
pub use pull::*;
pub use push::*;
pub use utils::{is_wick_package_reference, parse_reference, parse_reference_and_protocol};

use crate::error::OciError;

#[macro_use]
extern crate tracing;

// TODO: take auth as arg instead of env.
/// The ENV variable holding the OCI username.
pub const OCI_VAR_USER: &str = "OCI_USERNAME";
/// The ENV variable holding the OCI password.
pub const OCI_VAR_PASSWORD: &str = "OCI_PASSWORD";

const WASM_MEDIA_TYPE: &str = oci_distribution::manifest::WASM_LAYER_MEDIA_TYPE;
const LAYER_MEDIA_TYPE: &str = oci_distribution::manifest::IMAGE_LAYER_MEDIA_TYPE;

/// Retrieve a payload from an OCI url.
pub async fn fetch_oci_bytes(img: &str, allow_latest: bool, allowed_insecure: &[String]) -> Result<Vec<u8>, OciError> {
  if !allow_latest && img.ends_with(":latest") {
    return Err(OciError::LatestDisallowed(img.to_owned()));
  }
  debug!(image = img, "oci remote");

  let img = parse_reference(img)?;

  let auth = std::env::var(OCI_VAR_USER).map_or(oci_distribution::secrets::RegistryAuth::Anonymous, |u| {
    std::env::var(OCI_VAR_PASSWORD).map_or(oci_distribution::secrets::RegistryAuth::Anonymous, |p| {
      oci_distribution::secrets::RegistryAuth::Basic(u, p)
    })
  });

  let protocol = oci_distribution::client::ClientProtocol::HttpsExcept(allowed_insecure.to_vec());
  let config = oci_distribution::client::ClientConfig {
    protocol,
    ..Default::default()
  };
  let mut c = oci_distribution::Client::new(config);
  let imgdata = pull(&mut c, &img, &auth).await;

  match imgdata {
    Ok(imgdata) => {
      let content = imgdata.layers.into_iter().flat_map(|l| l.data).collect::<Vec<_>>();

      Ok(content)
    }
    Err(e) => {
      error!("Failed to fetch OCI bytes: {}", e);
      Err(OciError::OciFetchFailure(img.to_string(), e.to_string()))
    }
  }
}
