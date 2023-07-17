use wick_config::config::{ExecutionSettings, LiquidJsonConfig};
use wick_packet::{InherentData, RuntimeConfig};

use crate::error::InterpreterError;

#[derive(Debug, Clone, Default)]
pub struct OperationSettings {
  pub(crate) config: LiquidOperationConfig,
  pub(crate) settings: Option<ExecutionSettings>,
}

impl OperationSettings {
  /// Initialize a new OperationSettings with the specified config and settings.
  pub(crate) fn new(config: LiquidOperationConfig, settings: Option<ExecutionSettings>) -> Self {
    Self { config, settings }
  }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct LiquidOperationConfig {
  root: Option<RuntimeConfig>,
  template: Option<LiquidJsonConfig>,
  value: Option<RuntimeConfig>,
}

impl LiquidOperationConfig {
  #[must_use]
  pub(crate) fn new_value(value: Option<RuntimeConfig>) -> Self {
    Self {
      template: None,
      value,
      root: None,
    }
  }

  pub(crate) fn render(&self, inherent: &InherentData) -> Result<Option<RuntimeConfig>, InterpreterError> {
    if let Some(template) = self.template() {
      Ok(Some(
        template
          .render(self.root.as_ref(), self.value.as_ref(), None, Some(inherent))
          .map_err(|e| InterpreterError::Configuration(e.to_string()))?,
      ))
    } else {
      Ok(self.value.clone())
    }
  }

  #[must_use]
  pub(crate) fn value(&self) -> Option<&RuntimeConfig> {
    self.value.as_ref()
  }

  #[must_use]
  pub(crate) fn template(&self) -> Option<&LiquidJsonConfig> {
    self.template.as_ref()
  }

  #[must_use]
  pub(crate) fn root(&self) -> Option<&RuntimeConfig> {
    self.root.as_ref()
  }

  pub(crate) fn set_root(&mut self, root: Option<RuntimeConfig>) {
    self.root = root;
  }

  pub(crate) fn set_template(&mut self, template: Option<LiquidJsonConfig>) {
    self.template = template;
  }

  pub(crate) fn set_value(&mut self, value: Option<RuntimeConfig>) {
    self.value = value;
  }
}

impl From<Option<LiquidJsonConfig>> for LiquidOperationConfig {
  fn from(value: Option<LiquidJsonConfig>) -> Self {
    LiquidOperationConfig {
      template: value,
      value: None,
      root: None,
    }
  }
}

impl From<Option<RuntimeConfig>> for LiquidOperationConfig {
  fn from(value: Option<RuntimeConfig>) -> Self {
    LiquidOperationConfig {
      template: None,
      value,
      root: None,
    }
  }
}
