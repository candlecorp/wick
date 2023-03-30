use assets::AssetManager;

use crate::config;

#[derive(Debug, Clone, derive_assets::AssetManager)]
#[asset(crate::config::LocationReference)]
#[must_use]
pub struct TestConfiguration {
  #[asset(skip)]
  pub(crate) source: Option<String>,
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
  pub fn set_source(&mut self, source: impl AsRef<str>) {
    self.source = Some(source.as_ref().to_owned());
    self.set_baseurl(source.as_ref());
  }
}
