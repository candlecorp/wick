use asset_container::AssetManager;

use crate::config;

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(crate::config::AssetReference)]
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
  pub fn set_source(&mut self, source: String) {
    // Source is a file, so our baseurl needs to be the parent directory.
    // Remove the trailing filename from source.
    if source.ends_with(std::path::MAIN_SEPARATOR) {
      self.set_baseurl(&source);
      self.source = Some(source);
    } else {
      let s = source.rfind('/').map_or(source.as_str(), |index| &source[..index]);

      self.set_baseurl(s);
      self.source = Some(s.to_owned());
    }
  }
}
