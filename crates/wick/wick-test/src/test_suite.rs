use flow_component::SharedComponent;
use tap_harness::TestRunner;
use wick_config::config::TestCase;

use crate::{get_description, run_test, TestError, UnitTest};

#[derive(Debug)]
#[must_use]
pub struct TestSuite<'a> {
  tests: Vec<UnitTest<'a>>,
  name: String,
  filters: Vec<String>,
}

impl<'a> Default for TestSuite<'a> {
  fn default() -> Self {
    Self::new("Test")
  }
}

impl<'a> TestSuite<'a> {
  pub fn new<T: AsRef<str>>(name: T) -> Self {
    Self {
      tests: Vec::new(),
      name: name.as_ref().to_owned(),
      filters: Vec::new(),
    }
  }

  pub fn from_test_cases<'b>(tests: &'b [TestCase]) -> Self
  where
    'b: 'a,
  {
    let defs: Vec<UnitTest<'b>> = tests
      .iter()
      .map(|test| UnitTest {
        test,
        actual: Vec::new(),
      })
      .collect();
    Self {
      tests: defs,
      ..Default::default()
    }
  }

  pub fn get_tests<'b>(&'a mut self) -> Vec<&'a mut UnitTest<'b>>
  where
    'a: 'b,
  {
    let filters = &self.filters;
    if !filters.is_empty() {
      self
        .tests
        .iter_mut()
        .filter(|test| filters.iter().any(|filter| get_description(test).contains(filter)))
        .collect()
    } else {
      self.tests.iter_mut().collect()
    }
  }

  pub fn filter(mut self, filters: Vec<String>) -> Self {
    self.filters = filters;
    self
  }

  pub fn name(mut self, name: String) -> Self {
    self.name = name;
    self
  }

  pub async fn run(
    &'a mut self,
    component_id: Option<&str>,
    component: SharedComponent,
  ) -> Result<TestRunner, TestError> {
    let name = self.name.clone();
    let tests = self.get_tests();
    run_test(name, tests, component_id, component).await
  }
}
