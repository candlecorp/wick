#![allow(clippy::option_if_let_else)]
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, getset::Getters)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// A struct containing global and relative cache directories.
pub struct Directories {
  #[getset(get = "pub")]
  /// The root data directory.
  root: PathBuf,
  #[getset(get = "pub")]
  /// The directory to find and store processed data.
  cache: PathBuf,
  #[getset(get = "pub")]
  /// The directory to find and store downloaded or unprocessed artifacts.
  staging: PathBuf,
}

impl Directories {
  pub(crate) fn new(root: &Path) -> Self {
    Self {
      root: root.to_path_buf(),
      cache: root.join(crate::directory::CACHE),
      staging: root.join(crate::directory::STAGING),
    }
  }
}
