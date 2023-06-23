use flow_component::BoxFuture;
use tap_harness::TestRunner;
use wick_config::config::{TestCase, TestConfiguration};
use wick_packet::RuntimeConfig;

use crate::utils::render_config;
use crate::{run_test, TestError, UnitTest};

pub type ComponentFactory<'a> =
  Box<dyn Fn(Option<RuntimeConfig>) -> BoxFuture<'a, Result<SharedComponent, TestError>> + Sync + Send>;

pub use flow_component::SharedComponent;

#[derive(Debug, Default)]
#[must_use]
pub struct TestSuite<'a> {
  tests: Vec<TestGroup<'a>>,
}

impl<'a> TestSuite<'a> {
  pub fn from_configuration<'b>(configurations: &'b [TestConfiguration]) -> Result<Self, TestError>
  where
    'b: 'a,
  {
    let defs: Vec<TestGroup<'b>> = configurations
      .iter()
      .map(|config| {
        Ok(TestGroup::from_test_cases(
          render_config(config.config())?,
          config.cases(),
        ))
      })
      .collect::<Result<_, _>>()?;
    Ok(Self { tests: defs })
  }

  pub fn add_configuration<'b>(&mut self, config: &'b TestConfiguration) -> Result<(), TestError>
  where
    'b: 'a,
  {
    self.tests.push(TestGroup::from_test_cases(
      render_config(config.config())?,
      config.cases(),
    ));
    Ok(())
  }

  pub async fn run(&'a mut self, factory: ComponentFactory<'a>) -> Result<Vec<TestRunner>, TestError> {
    let mut runners = Vec::new();
    for group in &mut self.tests {
      let component = factory(group.root_config.clone());
      runners.push(group.run(None, component.await?).await?);
    }
    Ok(runners)
  }
}

#[derive(Debug)]
#[must_use]
pub struct TestGroup<'a> {
  tests: Vec<UnitTest<'a>>,
  root_config: Option<RuntimeConfig>,
  name: String,
}

impl<'a> TestGroup<'a> {
  pub fn from_test_cases<'b>(root_config: Option<RuntimeConfig>, tests: &'b [TestCase]) -> Self
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
      root_config,
      name: "Test".to_owned(),
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
    let config = self.root_config.clone();
    let tests = self.get_tests();
    run_test(name, tests, component_id, component, config).await
  }
}
