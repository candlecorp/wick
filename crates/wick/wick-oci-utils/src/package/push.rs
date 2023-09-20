use std::collections::HashMap;

use oci_distribution::client::{Client, ClientConfig, ImageLayer};
use oci_distribution::manifest::{OciDescriptor, OciImageManifest};
use oci_distribution::Reference;
use sha256::digest;

use super::annotations::Annotations;
use super::{annotations, media_types, PackageFile};
use crate::{Error, OciOptions, WickPackageKind};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PushResponse {
  /// Pullable url for the config
  pub config_url: String,
  /// Pullable url for the manifest
  pub manifest_url: String,
  /// The OCI reference for the pushed image
  pub reference: String,
  /// Pullable url for any additional tags referencing this manifest
  pub tags: Vec<String>,
}

/// Push a Wick package to a registry.
pub async fn push(
  reference: &str,
  kind: WickPackageKind,
  config_json: String,
  files: Vec<PackageFile>,
  annotations: Annotations,
  tags: Vec<String>,
  options: &OciOptions,
) -> Result<PushResponse, Error> {
  let mut layers: Vec<OciDescriptor> = Vec::new();
  let mut image_layers: Vec<ImageLayer> = Vec::new();

  for file in files {
    let mut annotations_map: HashMap<String, String> = HashMap::new();

    annotations_map.insert(annotations::TITLE.to_owned(), file.package_path().display().to_string());
    let media_type = file.media_type().to_owned();
    let digest = file.hash().to_owned();
    let data = file.into_contents();
    let data_len = data.len();
    let image_layer = ImageLayer {
      media_type: media_type.clone(),
      data: data.into(),
      annotations: None,
    };
    trace!(annotations=?annotations_map, "adding layer");

    let image_layer_descriptor = OciDescriptor {
      media_type,
      digest,
      size: data_len as i64,
      annotations: Some(annotations_map),
      urls: None,
    };

    layers.push(image_layer_descriptor);
    image_layers.push(image_layer);
  }

  let image_annotations: HashMap<String, String> = annotations.inner().clone();

  let (image_config, image_manifest) = gen_manifest(
    &config_json,
    layers,
    Some(image_annotations),
    Some(kind.media_type().to_owned()),
  );

  let (image_ref, protocol) = crate::utils::parse_reference_and_protocol(reference, &options.allow_insecure)?;
  let client_config = ClientConfig {
    protocol,
    ..Default::default()
  };

  let mut client = Client::new(client_config);
  let auth = options.get_auth();

  let push_response = match client
    .push(
      &image_ref,
      &image_layers,
      image_config,
      &auth,
      Some(image_manifest.clone()),
    )
    .await
  {
    Ok(push_response) => push_response,
    Err(e) => {
      tracing::error!(manifest = %image_ref, error = %e, "Push failed");
      return Err(Error::PushFailed(e.to_string()));
    }
  };

  let mut pushed_tags = Vec::new();
  for tag in tags {
    let image_ref = Reference::with_tag(image_ref.registry().to_owned(), image_ref.repository().to_owned(), tag);

    client
      .push_manifest(&image_ref, &(image_manifest.clone().into()))
      .await?;
    pushed_tags.push(image_ref.to_string());
  }

  Ok(PushResponse {
    config_url: push_response.config_url,
    manifest_url: push_response.manifest_url,
    reference: image_ref.to_string(),
    tags: pushed_tags,
  })
}

fn gen_manifest(
  config_json: &str,
  layers: Vec<OciDescriptor>,
  annotations: Option<HashMap<String, String>>,
  artifact_type: Option<String>,
) -> (oci_distribution::client::Config, OciImageManifest) {
  let image_config = oci_distribution::client::Config {
    data: config_json.as_bytes().to_vec(),
    media_type: media_types::CONFIG.to_owned(),
    annotations: None,
  };

  let manifest = OciImageManifest {
    schema_version: 2,
    config: OciDescriptor {
      media_type: image_config.media_type.clone(),
      digest: format!("sha256:{}", digest(config_json.clone())),
      size: image_config.data.len() as i64,
      annotations: None,
      urls: None,
    },
    layers,
    media_type: Some(media_types::MANIFEST.to_owned()),
    annotations,
    artifact_type,
  };
  (image_config, manifest)
}
