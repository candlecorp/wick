use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use flate2::write::GzEncoder;
use flate2::Compression;
use tar::Builder;
use tokio::fs::{self};
use wick_config::config::Metadata;
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

pub(crate) fn get_relative_path(base_dir: &PathBuf, path: &str) -> Result<PathBuf, Error> {
  let path = PathBuf::from(path);
  // Get the prefix of the path that matches the base directory
  let relative_part = path
    .strip_prefix(base_dir)
    .map_err(|_| Error::InvalidFileLocation(path.display().to_string()))?;

  // Return the relative path
  Ok(relative_part.to_path_buf())
}

pub(crate) async fn create_tar_gz(extra_files: Vec<String>, parent_dir: &PathBuf) -> Result<Vec<u8>, Error> {
  let mut tar_bytes = Vec::new();
  let mut tar = Builder::new(GzEncoder::new(&mut tar_bytes, Compression::default()));

  for file_path in extra_files {
    let path = parent_dir.join(file_path);
    let absolute_path = path.canonicalize().unwrap();
    //ensure that the file is within the parent directory
    get_relative_path(&parent_dir, absolute_path.to_str().unwrap()).unwrap();

    let metadata = fs::metadata(&path).await.map_err(|e| Error::TarFile(path.clone(), e))?;

    if metadata.is_file() {
      let mut file = File::open(&path).map_err(|e| Error::TarFile(path.clone(), e))?;
      let rel_path = path.strip_prefix(parent_dir).unwrap();
      tar
        .append_file(rel_path, &mut file)
        .map_err(|e| Error::TarFile(path.clone(), e))?;
    } else if metadata.is_dir() {
      let rel_path = path.strip_prefix(parent_dir).unwrap();
      tar
        .append_dir_all(rel_path, &path)
        .map_err(|e| Error::TarFile(path.clone(), e))?;
    }
  }

  drop(tar);

  Ok(tar_bytes)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_ensure_relative_path() {
    let parent_dir = PathBuf::from("/candlecorp/wick/crates/wick/wick-package/tests/files");
    let path = "/candlecorp/wick/crates/wick/wick-package/tests/files/assets/test.fake.wasm";

    let result = get_relative_path(&parent_dir, path);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("assets/test.fake.wasm"));
  }
}
