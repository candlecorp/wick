use std::collections::HashMap;

use oci_distribution::client::{Client, ClientConfig, ImageLayer, PushResponse};
use oci_distribution::manifest::{OciDescriptor, OciImageManifest};
use sha256::digest;

use super::annotations::Annotations;
use super::{annotations, media_types, PackageFile};
use crate::{Error, OciOptions};
/// Push a Wick package to a registry.
pub async fn push(
  reference: &str,
  config_json: String,
  files: Vec<PackageFile>,
  annotations: Annotations,
  options: &OciOptions,
) -> Result<PushResponse, Error> {
  let image_config = oci_distribution::client::Config {
    data: config_json.as_bytes().to_vec(),
    media_type: media_types::CONFIG.to_owned(),
    annotations: None,
  };

  let mut image_layer_descriptors: Vec<OciDescriptor> = Vec::new();
  let mut image_layers: Vec<ImageLayer> = Vec::new();

  for file in files {
    let mut annotations_map: HashMap<String, String> = HashMap::new();

    annotations_map.insert(annotations::TITLE.to_owned(), file.path().display().to_string());
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

    image_layer_descriptors.push(image_layer_descriptor);
    image_layers.push(image_layer);
  }

  let image_annotations: HashMap<String, String> = annotations.inner().clone();

  let image_manifest = OciImageManifest {
    schema_version: 2,
    config: OciDescriptor {
      media_type: image_config.media_type.clone(),
      digest: format!("sha256:{}", digest(config_json.clone())),
      size: image_config.data.clone().len() as i64,
      annotations: None,
      urls: None,
    },
    layers: image_layer_descriptors,
    media_type: Some(media_types::MANIFEST.to_owned()),
    annotations: Some(image_annotations),
  };

  let (image_ref, protocol) = crate::utils::parse_reference_and_protocol(reference, &options.allow_insecure)?;
  let client_config = ClientConfig {
    protocol,
    ..Default::default()
  };
  let mut client = Client::new(client_config);
  let auth = options.get_auth();

  let result = client
    .push(
      &image_ref,
      &image_layers,
      image_config,
      &auth,
      Some(image_manifest.clone()),
    )
    .await;

  match result {
    Ok(push_response) => Ok(push_response),
    Err(e) => {
      tracing::error!(manifest = %image_manifest, error = %e, "Push failed");
      Err(Error::PushFailed(e.to_string()))
    }
  }
}
