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
use oci_distribution::client::{ImageData, ImageLayer};
use oci_distribution::manifest::{self, ImageIndexEntry, OciImageIndex, OciImageManifest, OciManifest, Platform};
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::{Client, Reference};

#[macro_use]
extern crate tracing;

/// The ENV variable holding the OCI username.
pub const OCI_VAR_USER: &str = "OCI_REGISTRY_USER";
/// The ENV variable holding the OCI password.
pub const OCI_VAR_PASSWORD: &str = "OCI_REGISTRY_PASSWORD";

// const MANIFEST_MEDIA_TYPE: &str = "application/vnd.docker.distribution.manifest.v2+json";
// const MANIFEST_LIST_MEDIA_TYPE: &str = "application/vnd.docker.distribution.manifest.list.v2+json";
const WASM_MEDIA_TYPE: &str = oci_distribution::manifest::WASM_LAYER_MEDIA_TYPE;
const OCI_MEDIA_TYPE: &str = oci_distribution::manifest::IMAGE_LAYER_MEDIA_TYPE;
// const OCI_CONFIG_MEDIA_TYPE: &str = oci_distribution::manifest::IMAGE_CONFIG_MEDIA_TYPE;

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
    let img = parse_reference(img)?;

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
      ..Default::default()
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

async fn pull(client: &mut Client, img: &Reference, auth: &RegistryAuth) -> Result<ImageData, OciError> {
  client
    .pull(img, auth, vec![WASM_MEDIA_TYPE, OCI_MEDIA_TYPE])
    .await
    .map_err(|e| OciError::OciFetchFailure(img.to_string(), e.to_string()))
}

/// TODO
pub async fn push_multi_arch<T, U>(
  client: &mut Client,
  auth: &RegistryAuth,
  registry: T,
  repo: U,
  tag: Option<String>,
  arches: ArchitectureMap,
) -> Result<String, OciError>
where
  T: AsRef<str> + Send + Sync,
  U: AsRef<str> + Send + Sync,
{
  let mut root_manifest = OciImageIndex {
    schema_version: 2,
    media_type: Some(manifest::IMAGE_MANIFEST_LIST_MEDIA_TYPE.to_owned()),
    manifests: vec![],
  };

  for entry in arches {
    let os = entry.os.clone();
    let arch = entry.arch.clone();
    let (manifest, digest) = push_arch(client, auth, registry.as_ref(), repo.as_ref(), tag.clone(), entry).await?;
    let size = manifest.layers.iter().fold(0, |acc, next| acc + next.size);
    root_manifest.manifests.push(ImageIndexEntry {
      media_type: manifest
        .media_type
        .unwrap_or_else(|| "application/vnd.docker.distribution.manifest.v2+json".to_owned()),
      digest,
      size,
      platform: Some(Platform {
        architecture: arch,
        os,
        os_version: None,
        os_features: None,
        variant: None,
        features: None,
      }),
      annotations: None,
    });
  }
  debug!("OCI:MANIFEST_LIST:{}", root_manifest);

  let reference = parse_reference(&format!("{}/{}", registry.as_ref(), repo.as_ref()))?;

  let result = client
    .push_manifest_list(&reference, auth, root_manifest)
    .await
    .map_err(|e| OciError::OciPushManifestListFailure(reference, e.into()))?;
  Ok(result)
}

/// TODO
pub async fn push_arch<T, U>(
  client: &mut Client,
  auth: &RegistryAuth,
  registry: T,
  repo: U,
  tag: Option<String>,
  entry: ArchitectureEntry,
) -> Result<(OciImageManifest, String), OciError>
where
  T: AsRef<str> + Send,
  U: AsRef<str> + Send,
{
  let config = serde_json::json!({
    "architecture": entry.arch,
    "os": entry.os
  });

  let archrepo = format!("{}-{}-{}", repo.as_ref(), entry.os, entry.arch);

  let image_ref = match tag {
    Some(t) => Reference::with_tag(registry.as_ref().to_owned(), archrepo.clone(), t),
    None => Reference::with_tag(registry.as_ref().to_owned(), archrepo.clone(), "latest".to_owned()),
  };
  let image_data = ImageData {
    layers: vec![ImageLayer {
      data: entry.bytes,
      media_type: manifest::IMAGE_LAYER_MEDIA_TYPE.to_owned(),
    }],
    digest: None,
  };

  let response = client
    .push(
      &image_ref,
      &image_data,
      config.to_string().as_bytes(),
      &entry.media_type,
      auth,
      None,
    )
    .await
    .map_err(|e| OciError::OciPushFailure(image_ref.clone(), e.into()))?;
  let short_ref = format!("{}/{}", image_ref.registry(), image_ref.repository());
  debug!("OCI:PUSH:[{}][IMAGE_URL={}]", short_ref, response.image_url);
  debug!("OCI:PUSH:[{}][MANIFEST_URL={}]", short_ref, response.manifest_url);
  debug!("OCI:PUSH:[{}][IMAGE_URL={}]", short_ref, response.config_url);

  let (manifest, digest) = client
    .pull_manifest(&image_ref, auth)
    .await
    .map_err(|e| OciError::OciPullManifestFailure(image_ref, e.into()))?;

  let reference = Reference::with_digest(registry.as_ref().to_owned(), repo.as_ref().to_owned(), digest.clone());

  let response = client
    .push(
      &reference,
      &image_data,
      config.to_string().as_bytes(),
      &entry.media_type,
      auth,
      None,
    )
    .await
    .map_err(|e| OciError::OciPushFailure(reference.clone(), e.into()))?;

  let short_ref = format!("{}/{}", reference.registry(), reference.repository());
  debug!("OCI:PUSH:[{}][IMAGE_URL={}]", short_ref, response.image_url);
  debug!("OCI:PUSH:[{}][MANIFEST_URL={}]", short_ref, response.manifest_url);
  debug!("OCI:PUSH:[{}][IMAGE_URL={}]", short_ref, response.config_url);

  match manifest {
    OciManifest::Image(v) => Ok((v, digest)),
    OciManifest::ImageIndex(_) => unreachable!(),
  }
}

fn parse_reference(reference: &str) -> Result<Reference, OciError> {
  oci_distribution::Reference::from_str(reference)
    .map_err(|e| OciError::OCIParseError(reference.to_owned(), e.to_string()))
}

/// Entries in the ArchitectureMap
#[derive(Debug)]
pub struct ArchitectureEntry {
  os: String,
  arch: String,
  bytes: Vec<u8>,
  media_type: String,
}

/// Architecture map struct holds architectures for multi-arch push.
#[derive(Debug, Default)]
pub struct ArchitectureMap {
  arches: Vec<ArchitectureEntry>,
}

impl IntoIterator for ArchitectureMap {
  type Item = ArchitectureEntry;

  type IntoIter = Box<dyn Iterator<Item = Self::Item> + Sync + Send>;

  fn into_iter(self) -> Self::IntoIter {
    Box::new(self.arches.into_iter())
  }
}

impl ArchitectureMap {
  /// Add an architecture to the [ArchitectureMap]
  pub fn add<T, U>(&mut self, os: T, arch: U, bytes: Vec<u8>, media_type: Option<String>)
  where
    T: AsRef<str>,
    U: AsRef<str>,
  {
    let media_type = match media_type {
      Some(v) => v,
      None => manifest::IMAGE_CONFIG_MEDIA_TYPE.to_owned(),
    };
    self.arches.push(ArchitectureEntry {
      os: os.as_ref().to_owned(),
      arch: arch.as_ref().to_owned(),
      bytes,
      media_type,
    });
  }

  /// Create an iterator over the contained [ArchitectureEntry]s
  pub fn iter(&self) -> impl Iterator<Item = &ArchitectureEntry> {
    self.arches.iter()
  }
}

#[cfg(test)]
mod tests {

  use std::time::SystemTime;

  use anyhow::Result;
  use oci_distribution::client::{ClientConfig, ClientProtocol};
  use oci_distribution::secrets::RegistryAuth;

  use super::*;

  const REGISTRY: &str = "127.0.0.1:5000";

  #[test_log::test(tokio::test)]
  async fn test_push_multi_arch() -> Result<()> {
    let protocol = ClientProtocol::Http;
    let config = ClientConfig {
      protocol,
      ..Default::default()
    };
    let mut c = Client::new(config);
    let auth = RegistryAuth::Anonymous;
    let mut arches = ArchitectureMap::default();
    arches.add("windows", "amd64", b"win64".to_vec(), None);
    let now = SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)?
      .as_millis()
      .to_string()
      .as_bytes()
      .to_vec();
    println!("Publishing artifact with bytes: {:?}", now);
    arches.add("linux", "amd64", now.clone(), None);
    let repo = "test/multi-arch";

    let manifest_url = push_multi_arch(&mut c, &auth, REGISTRY, repo, None, arches).await?;
    println!("{}", manifest_url);
    let reference = Reference::with_tag(REGISTRY.to_owned(), repo.to_owned(), "latest".to_owned());

    let (manifest, config_digest, config) = c.pull_manifest_and_config(&reference, &auth).await?;
    println!("{}", manifest);
    println!("{}", config_digest);
    println!("{}", config);
    assert_eq!(config, r#"{"architecture":"amd64","os":"linux"}"#);
    let layers = c
      .pull(&reference, &auth, vec![manifest::IMAGE_LAYER_MEDIA_TYPE])
      .await?;

    assert_eq!(layers.layers.len(), 1);
    println!("Artifact has bytes: {:?}", layers.layers[0].data);
    println!("{}", config);
    assert_eq!(layers.layers[0].data, now);

    Ok(())
  }
}
