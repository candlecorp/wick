//! OCI fetch and utility package

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
#![allow(missing_docs)]

/// This crate's error module.
pub mod error;
mod manifest;
mod options;
/// OCI utilities related to pushing and pulling Wick packages.
pub mod package;
mod pull;
mod push;
pub mod utils;

pub use error::OciError as Error;
pub use manifest::*;
pub use oci_distribution::client::ClientProtocol;
pub use oci_distribution::manifest::{OciDescriptor, OciImageIndex, OciImageManifest, OciManifest};
pub use options::*;
pub use pull::*;
pub use push::*;
use serde::{Deserialize, Serialize};
pub use utils::{
  get_cache_directory,
  is_oci_reference,
  is_wick_package_reference,
  parse_reference,
  parse_reference_and_protocol,
};

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

/// This is the oci manifest config for Wick packages.
#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WickOciConfig {
  pub kind: WickPackageKind,
  pub root: String,
}

impl WickOciConfig {
  /// Create a new WickOciConfig.
  #[must_use]
  pub const fn new(kind: WickPackageKind, root: String) -> Self {
    Self { kind, root }
  }
}

/// Represents the kind of Wick package.
/// This is used to determine how to handle the package.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum WickPackageKind {
  /// A Wick application package.
  APPLICATION,
  /// A Wick component package.
  COMPONENT,
  /// A Wick types package.
  TYPES,
}

impl WickPackageKind {
  /// Get the media type for the kind.
  #[must_use]
  pub const fn media_type(&self) -> &'static str {
    use self::package::media_types;
    match self {
      Self::APPLICATION => media_types::APPLICATION,
      Self::COMPONENT => media_types::COMPONENT,
      Self::TYPES => media_types::TYPES,
    }
  }
}

/// Retrieve a manifest from an OCI url.
pub async fn fetch_image_manifest(image: &str, options: &OciOptions) -> Result<(OciImageManifest, String), OciError> {
  if !options.allow_latest && image.ends_with(":latest") {
    return Err(OciError::LatestDisallowed(image.to_owned()));
  }
  debug!(image, "oci remote");

  let image = parse_reference(image)?;

  let auth = options
    .username()
    .as_ref()
    .map_or(oci_distribution::secrets::RegistryAuth::Anonymous, |u| {
      options
        .password()
        .as_ref()
        .map_or(oci_distribution::secrets::RegistryAuth::Anonymous, |p| {
          oci_distribution::secrets::RegistryAuth::Basic(u.clone(), p.clone())
        })
    });

  let protocol = oci_distribution::client::ClientProtocol::HttpsExcept(options.allow_insecure.clone());
  let config = oci_distribution::client::ClientConfig {
    protocol,
    ..Default::default()
  };
  let mut client = oci_distribution::Client::new(config);
  let (manifest, digest) = client
    .pull_manifest(&image, &auth)
    .await
    .map_err(|e| OciError::OciFetchFailure(image.to_string(), e.to_string()))?;
  let OciManifest::Image(manifest) = manifest else {
    return Err(OciError::OciFetchFailure(
      image.to_string(),
      "manifest is not an image".to_owned(),
    ));
  };
  Ok((manifest, digest))
}
