use oci_distribution::client::{Config, ImageLayer, PushResponse};
use oci_distribution::manifest::{self};
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::{Client, Reference};

use crate::error::OciError;

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
