use oci_distribution::client::{Config, ImageLayer, PushResponse};
use oci_distribution::manifest::{self, ImageIndexEntry, OciImageIndex, OciImageManifest, OciManifest, Platform};
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::{Client, Reference};

use crate::architecture_map::{ArchitectureEntry, ArchitectureMap};
use crate::error::OciError;
use crate::parse_reference;

pub async fn push_multi_arch(
  client: &mut Client,
  auth: &RegistryAuth,
  reference: &Reference,
  arches: ArchitectureMap,
) -> Result<String, OciError> {
  let mut root_manifest = OciImageIndex {
    schema_version: 2,
    media_type: Some(manifest::IMAGE_MANIFEST_LIST_MEDIA_TYPE.to_owned()),
    manifests: vec![],
    annotations: None,
  };

  for entry in arches {
    let os = entry.os.clone();
    let arch = entry.arch.clone();
    let (manifest, digest) = push_arch(client, auth, reference, entry).await?;
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
  debug!(manifest = %root_manifest, "oci manifest list");

  let reference = parse_reference(&format!("{}/{}", reference.registry(), reference.repository()))?;

  let result = client
    .push_manifest_list(&reference, auth, root_manifest)
    .await
    .map_err(|e| OciError::OciPushManifestListFailure(reference, e.into()))?;
  Ok(result)
}

pub async fn push_arch(
  client: &mut Client,
  auth: &RegistryAuth,
  reference: &Reference,
  entry: ArchitectureEntry,
) -> Result<(OciImageManifest, String), OciError> {
  let config = serde_json::json!({
    "architecture": entry.arch,
    "os": entry.os
  });

  let config = Config::new(config.to_string().into_bytes(), entry.media_type.clone(), None);

  let archrepo = format!("{}-{}-{}", reference.repository(), entry.os, entry.arch);

  let image_ref = reference.tag().map_or_else(
    || Reference::with_tag(reference.registry().to_owned(), archrepo.clone(), "latest".to_owned()),
    |t| Reference::with_tag(reference.registry().to_owned(), archrepo.clone(), t.to_owned()),
  );
  let layers = vec![ImageLayer {
    data: entry.bytes,
    media_type: manifest::IMAGE_LAYER_MEDIA_TYPE.to_owned(),
    annotations: None,
  }];

  let response = client
    .push(&image_ref, &layers, config.clone(), auth, None)
    .await
    .map_err(|e| OciError::OciPushFailure(image_ref.clone(), e.into()))?;
  let short_ref = format!("{}/{}", image_ref.registry(), image_ref.repository());

  debug!(
    reference = %short_ref,
    url = %response.manifest_url,
    "push: oci manifest url"
  );
  debug!(
    reference = %short_ref,
    url = %response.config_url,
    "push: oci config url"
  );

  let (manifest, digest) = client
    .pull_manifest(&image_ref, auth)
    .await
    .map_err(|e| OciError::OciPullManifestFailure(image_ref, e.into()))?;

  let reference = Reference::with_digest(
    reference.registry().to_owned(),
    reference.repository().to_owned(),
    digest.clone(),
  );

  let response = client
    .push(&reference, &layers, config.clone(), auth, None)
    .await
    .map_err(|e| OciError::OciPushFailure(reference.clone(), e.into()))?;

  let short_ref = format!("{}/{}", reference.registry(), reference.repository());

  debug!(
    reference = %short_ref,
    url = %response.manifest_url,
    "push: oci manifest url"
  );
  debug!(
    reference = %short_ref,
    url = %response.config_url,
    "push: oci config url"
  );

  match manifest {
    OciManifest::Image(v) => Ok((v, digest)),
    OciManifest::ImageIndex(_) => unreachable!(),
  }
}

pub async fn push(
  client: &mut Client,
  auth: &RegistryAuth,
  reference: &Reference,
  layers: &[ImageLayer],
) -> Result<PushResponse, OciError> {
  let config = Config::new(b"{}".to_vec(), manifest::IMAGE_CONFIG_MEDIA_TYPE.to_owned(), None);

  client
    .push(reference, layers, config, auth, None)
    .await
    .map_err(|e| OciError::OciPushFailure(reference.clone(), e.into()))
}
