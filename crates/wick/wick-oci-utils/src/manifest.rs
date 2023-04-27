use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
pub struct AssetManifest {
  pub(crate) root: PathBuf,
  pub(crate) version: String,
}

impl AssetManifest {
  pub const FILENAME: &str = ".wick-manifest.json";
  pub fn new(root: PathBuf, version: String) -> Self {
    Self { root, version }
  }

  #[must_use]
  pub fn root(&self) -> &PathBuf {
    &self.root
  }

  #[must_use]
  pub fn version(&self) -> &String {
    &self.version
  }
}
