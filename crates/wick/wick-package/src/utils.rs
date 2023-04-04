use std::collections::HashMap;
use std::path::PathBuf;

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
