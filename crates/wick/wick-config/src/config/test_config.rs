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
    let mut source = source.to_path_buf();
    self.source = Some(source.clone());
    // Source is (should be) a file, so pop the filename before setting the baseurl.
    if !source.is_dir() {
      source.pop();
    }
    self.set_baseurl(&source);
  }
}
