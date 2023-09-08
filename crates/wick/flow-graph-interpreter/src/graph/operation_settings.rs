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
  pub(crate) const fn new(config: LiquidOperationConfig, settings: Option<ExecutionSettings>) -> Self {
    Self { config, settings }
  }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct LiquidOperationConfig {
  root: Option<RuntimeConfig>,
  template: Option<LiquidJsonConfig>,
  op_config: Option<RuntimeConfig>,
}

impl LiquidOperationConfig {
  pub(crate) fn render(&self, inherent: &InherentData) -> Result<Option<RuntimeConfig>, InterpreterError> {
    if let Some(template) = self.template() {
      Ok(Some(
        template
          .render(None, self.root.as_ref(), self.op_config.as_ref(), None, Some(inherent))
          .map_err(|e| InterpreterError::Configuration(e.to_string()))?,
      ))
    } else {
      Ok(self.op_config.clone())
    }
  }

  #[must_use]
  pub(crate) const fn op_config(&self) -> Option<&RuntimeConfig> {
    self.op_config.as_ref()
  }

  #[must_use]
  pub(crate) const fn template(&self) -> Option<&LiquidJsonConfig> {
    self.template.as_ref()
  }

  #[must_use]
  pub(crate) const fn root(&self) -> Option<&RuntimeConfig> {
    self.root.as_ref()
  }

  pub(crate) fn set_root(&mut self, root: Option<RuntimeConfig>) {
    self.root = root;
  }

  pub(crate) fn set_template(&mut self, template: Option<LiquidJsonConfig>) {
    self.template = template;
  }

  pub(crate) fn set_op_config(&mut self, config: Option<RuntimeConfig>) {
    self.op_config = config;
  }
}

impl From<Option<LiquidJsonConfig>> for LiquidOperationConfig {
  fn from(value: Option<LiquidJsonConfig>) -> Self {
    LiquidOperationConfig {
      template: value,
      op_config: None,
      root: None,
    }
  }
}

impl From<Option<RuntimeConfig>> for LiquidOperationConfig {
  fn from(value: Option<RuntimeConfig>) -> Self {
    LiquidOperationConfig {
      template: None,
      op_config: value,
      root: None,
    }
  }
}
