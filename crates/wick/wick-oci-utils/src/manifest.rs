use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
pub struct AssetManifest {
  pub(crate) root: PathBuf,
}

impl AssetManifest {
  pub const FILENAME: &str = ".wick-manifest.json";
  pub fn new(root: PathBuf) -> Self {
    Self { root }
  }

  #[must_use]
  pub fn root(&self) -> &PathBuf {
    &self.root
  }
}
