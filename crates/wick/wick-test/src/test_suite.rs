use tap_harness::TestRunner;
use wick_config::config::TestConfiguration;

use crate::utils::render_config;
use crate::{ComponentFactory, TestError, TestGroup};

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
          render_config(config.config(), None)?,
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
      render_config(config.config(), None)?,
      config.cases(),
    ));
    Ok(())
  }

  pub async fn run(
    &'a mut self,
    factory: ComponentFactory<'a>,
    filter: Vec<String>,
  ) -> Result<Vec<TestRunner>, TestError> {
    let mut runners = Vec::new();
    for group in &mut self.tests {
      let component = factory(group.root_config.clone());

      runners.push(group.run(None, component.await?, &filter).await?);
    }
    Ok(runners)
  }
}
