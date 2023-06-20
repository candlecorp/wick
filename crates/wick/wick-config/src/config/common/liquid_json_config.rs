use std::collections::HashMap;

use liquid_json::LiquidJsonValue;
use tracing::trace;
use wick_packet::RuntimeConfig;

use crate::Error;

// TODO:REFACTOR LiquidJsonConfig is too similar to the string-oriented TemplateConfig now.

/// A generic configuration that may include LiquidJson values
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LiquidJsonConfig {
  pub(crate) value: Option<RuntimeConfig>,
  pub(crate) template: HashMap<String, LiquidJsonValue>,
  pub(crate) root_config: Option<RuntimeConfig>,
}

impl LiquidJsonConfig {
  /// Make a template render context from the passed configuration.
  pub fn make_context(
    base: Option<serde_json::Value>,
    root: Option<&RuntimeConfig>,
    config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<serde_json::Value, Error> {
    trace!(root_config=?root, ?config, "rendering liquid config");
    let mut base = base.unwrap_or(serde_json::Value::Object(Default::default()));
    let map = base.as_object_mut().ok_or(Error::ContextBase)?;
    map.insert(
      "ctx".to_owned(),
      serde_json::json!({

          "root_config": root,
          "config": config,
          "env": env

      }),
    );
    Ok(base)
  }

  /// Render a [LiquidJsonConfig] into a [RuntimeConfig] creating a context from the passed configuration.
  pub fn render(
    &self,
    root: Option<&RuntimeConfig>,
    config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<RuntimeConfig, Error> {
    let ctx = Self::make_context(None, root, config, env)?;
    let mut map = HashMap::new();
    for (k, v) in self.template.iter() {
      map.insert(k.clone(), v.render(&ctx)?);
    }
    Ok(RuntimeConfig::from(map))
  }

  /// Render a [LiquidJsonConfig] into a [RuntimeConfig] with the passed context directly.
  pub fn render_raw(&self, ctx: &serde_json::Value) -> Result<RuntimeConfig, Error> {
    let ctx = serde_json::json!({
      "ctx": ctx,
    });

    let mut map = HashMap::new();
    for (k, v) in self.template.iter() {
      map.insert(k.clone(), v.render(&ctx)?);
    }
    Ok(RuntimeConfig::from(map))
  }

  #[must_use]
  /// Retrieve the runtime configuration
  pub fn value(&self) -> Option<&RuntimeConfig> {
    self.value.as_ref()
  }

  /// Set the runtime configuration
  pub fn set_value(&mut self, value: Option<RuntimeConfig>) {
    self.value = value;
  }
}

impl From<HashMap<String, LiquidJsonValue>> for LiquidJsonConfig {
  fn from(value: HashMap<String, LiquidJsonValue>) -> Self {
    Self {
      value: None,
      template: value,
      root_config: None,
    }
  }
}

impl From<HashMap<String, serde_json::Value>> for LiquidJsonConfig {
  fn from(value: HashMap<String, serde_json::Value>) -> Self {
    Self {
      template: value.into_iter().map(|(k, v)| (k, v.into())).collect(),
      value: None,
      root_config: None,
    }
  }
}

impl From<LiquidJsonConfig> for HashMap<String, LiquidJsonValue> {
  fn from(value: LiquidJsonConfig) -> Self {
    value.template
  }
}
