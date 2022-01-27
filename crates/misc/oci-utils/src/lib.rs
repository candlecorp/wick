//! OCI fetch and utility package

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
#![allow()]

use std::env::temp_dir;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

use crate::error::OciError;
/// This crate's error module.
pub mod error;
pub use error::OciError as Error;

#[macro_use]
extern crate tracing;

/// The ENV variable holding the OCI username.
pub const OCI_VAR_USER: &str = "OCI_REGISTRY_USER";
/// The ENV variable holding the OCI password.
pub const OCI_VAR_PASSWORD: &str = "OCI_REGISTRY_PASSWORD";

const WASM_MEDIA_TYPE: &str = "application/vnd.module.wasm.content.layer.v1+wasm";
const OCI_MEDIA_TYPE: &str = "application/vnd.oci.image.layer.v1.tar";

/// Retrieve a payload from an OCI url.
pub async fn fetch_oci_bytes(img: &str, allow_latest: bool, allowed_insecure: &[String]) -> Result<Vec<u8>, OciError> {
  if !allow_latest && img.ends_with(":latest") {
    return Err(OciError::LatestDisallowed(img.to_owned()));
  }
  let cf = cached_file(img);
  if cf.exists() {
    debug!("OCI:CACHE:{}", cf.to_string_lossy());
    let mut buf = vec![];
    let mut f = std::fs::File::open(cached_file(img))?;
    f.read_to_end(&mut buf)?;
    Ok(buf)
  } else {
    debug!("OCI:REMOTE:{}", img);
    let img =
      oci_distribution::Reference::from_str(img).map_err(|e| OciError::OCIParseError(img.to_owned(), e.to_string()))?;
    let auth = if let Ok(u) = std::env::var(OCI_VAR_USER) {
      if let Ok(p) = std::env::var(OCI_VAR_PASSWORD) {
        oci_distribution::secrets::RegistryAuth::Basic(u, p)
      } else {
        oci_distribution::secrets::RegistryAuth::Anonymous
      }
    } else {
      oci_distribution::secrets::RegistryAuth::Anonymous
    };

    let protocol = oci_distribution::client::ClientProtocol::HttpsExcept(allowed_insecure.to_vec());
    let config = oci_distribution::client::ClientConfig {
      protocol,
      accept_invalid_hostnames: false,
      accept_invalid_certificates: false,
      extra_root_certificates: vec![],
    };
    let mut c = oci_distribution::Client::new(config);
    let imgdata = pull(&mut c, &img, &auth).await;

    match imgdata {
      Ok(imgdata) => {
        let mut f = std::fs::File::create(cf)?;
        let content = imgdata.layers.into_iter().flat_map(|l| l.data).collect::<Vec<_>>();
        f.write_all(&content)?;
        f.flush()?;
        Ok(content)
      }
      Err(e) => {
        error!("Failed to fetch OCI bytes: {}", e);
        Err(OciError::OciFetchFailure(img.to_string(), e.to_string()))
      }
    }
  }
}

fn cached_file(img: &str) -> PathBuf {
  let path = temp_dir();
  let path = path.join("vino");
  let path = path.join("ocicache");
  let _ = ::std::fs::create_dir_all(&path);
  let img = img.replace(":", "_");
  let img = img.replace("/", "_");
  let img = img.replace(".", "_");
  let mut path = path.join(img);
  path.set_extension("bin");

  path
}

async fn pull(
  client: &mut oci_distribution::Client,
  img: &oci_distribution::Reference,
  auth: &oci_distribution::secrets::RegistryAuth,
) -> Result<oci_distribution::client::ImageData, OciError> {
  client
    .pull(img, auth, vec![WASM_MEDIA_TYPE, OCI_MEDIA_TYPE])
    .await
    .map_err(|e| OciError::OciFetchFailure(img.to_string(), e.to_string()))
}
