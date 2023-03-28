use crate::TestCase;

#[derive(Debug, Clone)]
#[must_use]
pub struct TestConfiguration {
  pub(crate) source: Option<String>,
  pub(crate) tests: Vec<TestCase>,
}

impl TestConfiguration {
  /// Return the list of tests defined in the manifest.
  #[must_use]
  pub fn tests(&self) -> &[TestCase] {
    &self.tests
  }
}