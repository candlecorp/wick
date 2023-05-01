use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use asset_container::Asset;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::Builder;
use tokio::fs::{self};
use wick_config::config::{AssetReference, Metadata};
use wick_oci_utils::package::annotations::{self, Annotations};

use crate::Error;

pub(crate) fn metadata_to_annotations(metadata: Metadata) -> Annotations {
  let mut map = HashMap::new();

  map.insert(annotations::VERSION.to_owned(), metadata.version.clone());

  if !metadata.authors.is_empty() {
    map.insert(annotations::AUTHORS.to_owned(), metadata.authors.join(", "));
  }

  if !metadata.vendors.is_empty() {
    map.insert(annotations::VENDORS.to_owned(), metadata.vendors.join(", "));
  }

  if let Some(description) = &metadata.description {
    map.insert(annotations::DESCRIPTION.to_owned(), description.clone());
  }

  if let Some(documentation) = &metadata.documentation {
    map.insert(annotations::DOCUMENTATION.to_owned(), documentation.clone());
  }

  if !metadata.licenses.is_empty() {
    map.insert(annotations::LICENSES.to_owned(), metadata.licenses.join(", "));
  }

  map.insert(
    annotations::ICON.to_owned(),
    metadata.icon.map(|v| v.path().unwrap_or_default()).unwrap_or_default(),
  );

  Annotations::new(map)
}

pub(crate) async fn create_tar_gz(extra_files: Vec<AssetReference>, parent_dir: &Path) -> Result<Vec<u8>, Error> {
  let mut tar_bytes = Vec::new();
  let mut tar = Builder::new(GzEncoder::new(&mut tar_bytes, Compression::default()));

  for file_path in extra_files {
    let absolute_path = file_path
      .path()
      .map_err(|_e| Error::NotFound(file_path.location().to_owned()))?;
    file_path.update_baseurl(parent_dir.to_str().unwrap());

    let relative_path = file_path.get_relative_part()?;

    let metadata = fs::metadata(&absolute_path)
      .await
      .map_err(|e| Error::TarFile(absolute_path.clone(), e))?;

    if metadata.is_file() {
      let mut file = File::open(&absolute_path).map_err(|e| Error::TarFile(absolute_path.clone(), e))?;
      tar
        .append_file(relative_path, &mut file)
        .map_err(|e| Error::TarFile(absolute_path.clone(), e))?;
    } else if metadata.is_dir() {
      tar
        .append_dir_all(relative_path, &absolute_path)
        .map_err(|e| Error::TarFile(absolute_path.clone(), e))?;
    }
  }

  drop(tar);

  Ok(tar_bytes)
}
