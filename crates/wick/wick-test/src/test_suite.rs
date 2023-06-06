use flow_component::BoxFuture;
use tap_harness::TestRunner;
use wick_config::config::{TestCase, TestConfiguration};
use wick_packet::GenericConfig;

use crate::{run_test, TestError, UnitTest};

pub type ComponentFactory<'a> =
  Box<dyn Fn(Option<GenericConfig>) -> BoxFuture<'a, Result<SharedComponent, TestError>> + Sync + Send>;

pub use flow_component::SharedComponent;

#[derive(Debug, Default)]
#[must_use]
pub struct TestSuite<'a> {
  tests: Vec<TestGroup<'a>>,
}

impl<'a> TestSuite<'a> {
  pub fn from_configuration<'b>(configurations: &'b [TestConfiguration]) -> Self
  where
    'b: 'a,
  {
    let defs: Vec<TestGroup<'b>> = configurations
      .iter()
      .map(|config| TestGroup::from_test_cases(config.config().cloned(), config.cases()))
      .collect();
    Self { tests: defs }
  }

  pub fn add_configuration<'b>(&mut self, config: &'b TestConfiguration)
  where
    'b: 'a,
  {
    self
      .tests
      .push(TestGroup::from_test_cases(config.config().cloned(), config.cases()));
  }

  pub async fn run(&'a mut self, factory: ComponentFactory<'a>) -> Result<Vec<TestRunner>, TestError> {
    let mut runners = Vec::new();
    for group in &mut self.tests {
      let component = factory(group.config.clone());
      runners.push(
        group
          .run(None, component.await.map_err(|e| TestError::Factory(e.to_string()))?)
          .await?,
      );
    }
    Ok(runners)
  }
}

#[derive(Debug)]
#[must_use]
pub struct TestGroup<'a> {
  tests: Vec<UnitTest<'a>>,
  config: Option<GenericConfig>,
  name: String,
}

impl<'a> Default for TestGroup<'a> {
  fn default() -> Self {
    Self {
      name: "Test".to_owned(),
      config: None,
      tests: Vec::new(),
    }
  }
}

impl<'a> TestGroup<'a> {
  pub fn new<T: AsRef<str>>(name: T) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      ..Default::default()
    }
  }

  pub fn from_test_cases<'b>(config: Option<GenericConfig>, tests: &'b [TestCase]) -> Self
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
      config,
      ..Default::default()
    }
  }

  pub fn get_tests<'b>(&'a mut self) -> Vec<&'a mut UnitTest<'b>>
  where
    'a: 'b,
  {
    self.tests.iter_mut().collect()
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
