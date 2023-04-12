use std::path::{Path, PathBuf};

use oci_distribution::manifest;
use serde::{Deserialize, Serialize};
use wick_grpctar::make_archive;
use wick_interface_types::ComponentSignature;
use wick_wascap::{ClaimsOptions, KeyPair};

use crate::error::OciError;

/// Entries in the ArchitectureMap
#[derive(Debug)]
pub struct ArchitectureEntry {
  pub(crate) os: String,
  pub(crate) arch: String,
  pub(crate) bytes: Vec<u8>,
  pub(crate) media_type: String,
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
    let media_type = media_type.map_or_else(|| manifest::IMAGE_CONFIG_MEDIA_TYPE.to_owned(), |v| v);
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

/// A manifest for a multi-architecture OCI artifact.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MultiArchManifest {
  /// The interface these artifacts conform to.
  pub interface: PathBuf,
  /// A list of tags to associate with this bundle.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tags: Option<Vec<String>>,
  /// A list of architectures and their artifacts
  pub artifacts: Vec<MultiArchManifestEntry>,
}

/// An entry for a specific architecture in a [MultiArchManifest].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MultiArchManifestEntry {
  /// The path to the artifact binary.
  pub path: PathBuf,
  /// The operating system this artifact is built for.
  pub os: String,
  /// The architecture this artifact is built for.
  pub arch: String,
}

/// Generate an [ArchitectureMap] from the passed directory.
pub async fn generate_archmap(
  manifest: &Path,
  options: ClaimsOptions,
  subject_kp: &KeyPair,
  issuer_kp: &KeyPair,
) -> Result<ArchitectureMap, OciError> {
  let basedir = manifest.parent().ok_or_else(|| {
    std::io::Error::new(
      std::io::ErrorKind::NotFound,
      format!("Could not find parent directory for {}", manifest.to_string_lossy()),
    )
  })?;
  debug!(
    path = %manifest.to_string_lossy(),
    "archmap manifest"
  );
  let bytes = tokio::fs::read(manifest).await?;
  let pushmanifest: MultiArchManifest = serde_yaml::from_slice(&bytes)?;
  let interface_path = basedir.join(pushmanifest.interface);
  debug!(
    path = %interface_path.to_string_lossy(),
    "archmap interface"
  );
  let interface_bytes = tokio::fs::read(interface_path).await?;
  let signature: ComponentSignature = serde_json::from_slice(&interface_bytes)?;

  let mut archmap = ArchitectureMap::default();
  for entry in pushmanifest.artifacts {
    let binary_path = basedir.join(entry.path);
    debug!(
      path = %binary_path.to_string_lossy(),
      "archmap binary"
    );
    let bin_bytes = std::fs::File::open(binary_path)?;

    let archive_bytes = make_archive(&bin_bytes, &signature, options.clone(), subject_kp, issuer_kp)?;

    archmap.add(entry.os, entry.arch, archive_bytes, None);
  }

  Ok(archmap)
}

#[cfg(test)]
mod integration_tests {

  use std::time::SystemTime;

  use anyhow::Result;
  use oci_distribution::client::{ClientConfig, ClientProtocol};
  use oci_distribution::secrets::RegistryAuth;
  use oci_distribution::{Client, Reference};

  use super::*;
  use crate::push_multi_arch;

  #[test_logger::test(tokio::test)]
  async fn integration_test_push_multi_arch() -> Result<()> {
    let registry = std::env::var("DOCKER_REGISTRY").unwrap();
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
    arches.add("darwin", "amd64", now.clone(), None);
    arches.add("darwin", "arm64", now.clone(), None);
    let repo = "test/multi-arch";
    let reference = Reference::with_tag(registry.clone(), repo.to_owned(), "latest".to_owned());

    let manifest_url = push_multi_arch(&mut c, &auth, &reference, arches).await?;
    println!("{}", manifest_url);

    let (manifest, config_digest, config) = c.pull_manifest_and_config(&reference, &auth).await?;
    println!("{}", manifest);
    println!("{}", config_digest);
    println!("{}", config);
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    assert_eq!(config, r#"{"architecture":"amd64","os":"linux"}"#);
    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    assert_eq!(config, r#"{"architecture":"arm64","os":"darwin"}"#);
    #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
    assert_eq!(config, r#"{"architecture":"amd64","os":"darwin"}"#);
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
