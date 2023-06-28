use std::collections::HashMap;

use liquid_json::LiquidJsonValue;
use tracing::{debug_span, trace};
use wick_packet::{date_from_millis, InherentData, RuntimeConfig};

use crate::Error;

// TODO:REFACTOR LiquidJsonConfig is too similar to the string-oriented TemplateConfig now.

/// A generic configuration that may include LiquidJson values
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LiquidJsonConfig {
  pub(crate) value: Option<RuntimeConfig>,
  pub(crate) template: HashMap<String, LiquidJsonValue>,
  pub(crate) root_config: Option<RuntimeConfig>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CtxInherent {
  timestamp: String,
}

impl LiquidJsonConfig {
  /// Make a template render context from the passed configuration.
  pub fn make_context(
    base: Option<serde_json::Value>,
    root: Option<&RuntimeConfig>,
    config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
    inherent: Option<&InherentData>,
  ) -> Result<serde_json::Value, Error> {
    trace!(root_config=?root, ?config, env=?env.map(|e|e.len()), inherent=?inherent, "rendering liquid config");
    let mut base = base.unwrap_or(serde_json::Value::Object(Default::default()));
    let map = base.as_object_mut().ok_or(Error::ContextBase)?;
    // The rust liquid template library is picky with the formats it recognizes.
    // This turns the timestamp into something it can use.
    // Datetime format: "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory][offset_minute]"
    let inherent = if let Some(i) = inherent {
      Some(CtxInherent {
        timestamp: date_from_millis(i.timestamp)
          .map(|d| d.format("%F %T %z").to_string())
          .map_err(|e| Error::ConfigurationTemplate(e.to_string()))?,
      })
    } else {
      None
    };

    map.insert(
      "ctx".to_owned(),
      serde_json::json!({
          "root_config": root,
          "config": config,
          "env": env,
          "inherent": inherent,
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
    inherent: Option<&InherentData>,
  ) -> Result<RuntimeConfig, Error> {
    let ctx = debug_span!("liquid-json-config").in_scope(|| Self::make_context(None, root, config, env, inherent))?;

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
