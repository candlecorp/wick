use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use vino_par::make_archive;
use vino_wascap::{ClaimsOptions, KeyPair};
use wasmflow_interface::ProviderSignature;

use crate::error::OciError;
use crate::ArchitectureMap;

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
  let signature: ProviderSignature = serde_json::from_slice(&interface_bytes)?;

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
