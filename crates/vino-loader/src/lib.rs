pub mod error;

pub(crate) type Result<T> = std::result::Result<T, error::LoadError>;
pub type Error = error::LoadError;

#[macro_use]
extern crate tracing;

use std::path::Path;

pub async fn get_bytes_from_file(path: &Path) -> Result<Vec<u8>> {
  Ok(tokio::fs::read(path).await?)
}

pub async fn get_bytes_from_oci(
  path: &str,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<Vec<u8>> {
  Ok(oci_utils::fetch_oci_bytes(path, allow_latest, allowed_insecure).await?)
}

pub async fn get_bytes(
  location: &str,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<Vec<u8>> {
  let path = Path::new(&location);
  if path.exists() {
    debug!("LOAD:AS_FILE:{}", location);
    Ok(get_bytes_from_file(path).await?)
  } else {
    debug!("LOAD:AS_OCI:{}", location);
    Ok(get_bytes_from_oci(location, allow_latest, allowed_insecure).await?)
  }
}
