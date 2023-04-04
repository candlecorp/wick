use oci_distribution::client::ImageData;
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::{Client, Reference};

use crate::error::OciError;
use crate::{LAYER_MEDIA_TYPE, WASM_MEDIA_TYPE};

pub async fn pull(client: &mut Client, img: &Reference, auth: &RegistryAuth) -> Result<ImageData, OciError> {
  client
    .pull(img, auth, vec![WASM_MEDIA_TYPE, LAYER_MEDIA_TYPE])
    .await
    .map_err(|e| OciError::OciFetchFailure(img.to_string(), e.to_string()))
}
