use assets::AssetManager;
use url::Url;

use crate::config;

#[derive(Debug, Clone, derive_assets::AssetManager)]
#[asset(crate::config::AssetReference)]
#[must_use]
pub struct TestConfiguration {
  #[asset(skip)]
  pub(crate) source: Option<Url>,
  #[asset(skip)]
  pub(crate) tests: Vec<config::TestCase>,
}

impl TestConfiguration {
  /// Return the list of tests defined in the manifest.
  #[must_use]
  pub fn tests(&self) -> &[config::TestCase] {
    &self.tests
  }

  /// Set the source location of the configuration.
  pub fn set_source(&mut self, source: Url) {
    // Source is a file, so our baseurl needs to be the parent directory.
    self.set_baseurl(source.join("./").unwrap().as_str());
    self.source = Some(source);
  }
}
