use std::path::{Path, PathBuf};

use asset_container::AssetManager;

use crate::config;

#[derive(Debug, Clone, Builder, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(crate::config::AssetReference))]
#[must_use]
pub struct TestConfiguration {
  #[asset(skip)]
  #[property(skip)]
  pub(crate) source: Option<PathBuf>,
  #[asset(skip)]
  pub(crate) tests: Vec<config::TestCase>,
}

impl TestConfiguration {
  /// Set the source location of the configuration.
  pub fn set_source(&mut self, source: &Path) {
    // Source is a file, so our baseurl needs to be the parent directory.
    // Remove the trailing filename from source.
    if source.is_dir() {
      self.set_baseurl(source);
      self.source = Some(source.to_path_buf());
    } else {
      let mut s = source.to_path_buf();
      s.pop();

      self.set_baseurl(&s);
      self.source = Some(s);
    }
  }
}
