use std::env::temp_dir;
use std::io::{
  Read,
  Write,
};
use std::path::PathBuf;
use std::str::FromStr;

use crate::error::OciError;

pub(crate) const OCI_VAR_USER: &str = "OCI_REGISTRY_USER";
pub(crate) const OCI_VAR_PASSWORD: &str = "OCI_REGISTRY_PASSWORD";
const PROVIDER_ARCHIVE_MEDIA_TYPE: &str = "application/vnd.wasmcloud.provider.archive.layer.v1+par";
const WASM_MEDIA_TYPE: &str = "application/vnd.module.wasm.content.layer.v1+wasm";
const OCI_MEDIA_TYPE: &str = "application/vnd.oci.image.layer.v1.tar";

pub(crate) async fn fetch_oci_bytes(
  img: &str,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<Vec<u8>, OciError> {
  if !allow_latest && img.ends_with(":latest") {
    return Err(OciError::LatestDisallowed(img.to_owned()));
  }
  let cf = cached_file(img);
  if !cf.exists() {
    let img = oci_distribution::Reference::from_str(img)
      .map_err(|e| OciError::OCIParseError(img.to_owned(), e.to_string()))?;
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
    let config = oci_distribution::client::ClientConfig { protocol };
    let mut c = oci_distribution::Client::new(config);
    let imgdata = pull(&mut c, &img, &auth).await;

    match imgdata {
      Ok(imgdata) => {
        let mut f = std::fs::File::create(cf)?;
        let content = imgdata
          .layers
          .iter()
          .flat_map(|l| l.data.clone())
          .collect::<Vec<_>>();
        f.write_all(&content)?;
        f.flush()?;
        Ok(content)
      }
      Err(e) => {
        error!("Failed to fetch OCI bytes: {}", e);
        Err(OciError::OciFetchFailure(img.to_string(), e.to_string()))
      }
    }
  } else {
    let mut buf = vec![];
    let mut f = std::fs::File::open(cached_file(img))?;
    f.read_to_end(&mut buf)?;
    Ok(buf)
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
    .pull(
      img,
      auth,
      vec![PROVIDER_ARCHIVE_MEDIA_TYPE, WASM_MEDIA_TYPE, OCI_MEDIA_TYPE],
    )
    .await
    .map_err(|e| OciError::OciFetchFailure(img.to_string(), e.to_string()))
}