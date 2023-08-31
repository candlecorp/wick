use flow_component::SharedComponent;
use tap_harness::TestRunner;
use wick_config::config::test_case::TestCase;
use wick_packet::RuntimeConfig;

use crate::{run_test, TestError, UnitTest};

#[derive(Debug)]
#[must_use]
pub struct TestGroup<'a> {
  pub(crate) tests: Vec<UnitTest<'a>>,
  pub(crate) root_config: Option<RuntimeConfig>,
  pub(crate) name: String,
}

impl<'a> TestGroup<'a> {
  pub fn from_test_cases<'b>(root_config: Option<RuntimeConfig>, tests: &'b [TestCase]) -> Self
  where
    'b: 'a,
  {
    let defs: Vec<UnitTest<'b>> = tests.iter().map(UnitTest::new).collect();
    Self {
      tests: defs,
      root_config,
      name: "Test".to_owned(),
    }
  }

  pub fn name(mut self, name: String) -> Self {
    self.name = name;
    self
  }

  pub async fn run(
    &'a mut self,
    component_id: Option<&str>,
    component: SharedComponent,
    filter: &[String],
  ) -> Result<TestRunner, TestError> {
    let name = self.name.clone();
    let config = self.root_config.clone();
    let tests = self
      .tests
      .iter_mut()
      .filter(|test| {
        if filter.is_empty() {
          return true;
        }
        test
          .test
          .name()
          .map_or(false, |name| filter.iter().any(|f| name.contains(f)))
      })
      .collect();
    run_test(name, tests, component_id, component, config).await
  }
}
