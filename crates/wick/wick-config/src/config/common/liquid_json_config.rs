use std::collections::HashMap;
use std::path::Path;

use liquid_json::LiquidJsonValue;
use serde_json::Value;
use tracing::trace;
use wick_packet::{date_from_millis, InherentData, RuntimeConfig};

use crate::Error;

// TODO:REFACTOR LiquidJsonConfig is too similar to the string-oriented TemplateConfig now.

/// A generic configuration that may include LiquidJson values
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize)]
pub struct LiquidJsonConfig {
  #[serde(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) value: Option<RuntimeConfig>,
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub(crate) template: HashMap<String, LiquidJsonValue>,
  #[serde(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) root_config: Option<RuntimeConfig>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CtxInherent {
  timestamp: String,
}

impl LiquidJsonConfig {
  /// Make a template render context from the passed configuration.
  pub fn make_context(
    base: Option<Value>,
    root: Option<&RuntimeConfig>,
    config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
    inherent: Option<&InherentData>,
  ) -> Result<Value, Error> {
    trace!(root_config=?root, ?config, env=?env.map(|e|e.len()), inherent=?inherent, "rendering liquid config");
    let mut base = base.unwrap_or(Value::Object(Default::default()));
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
    source: Option<&Path>,
    root: Option<&RuntimeConfig>,
    config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
    inherent: Option<&InherentData>,
  ) -> Result<RuntimeConfig, Error> {
    if let Some(value) = self.value.as_ref() {
      return Ok(value.clone());
    }

    let base = source.map(|source| {
      let dirname = source.parent().unwrap_or_else(|| Path::new("<unavailable>"));
      serde_json::json!({"__dirname": dirname})
    });

    let ctx = Self::make_context(base, root, config, env, inherent)?;

    let mut map = HashMap::new();
    for (k, v) in self.template.iter() {
      map.insert(k.clone(), v.render(&ctx)?);
    }
    Ok(RuntimeConfig::from(map))
  }

  /// Render a [LiquidJsonConfig] into a [RuntimeConfig] with the passed context directly.
  pub fn render_raw(&self, ctx: &Value) -> Result<RuntimeConfig, Error> {
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
  pub fn new_value(value: RuntimeConfig) -> Self {
    Self {
      value: Some(value),
      template: Default::default(),
      root_config: None,
    }
  }

  #[must_use]
  pub fn new_template(template: HashMap<String, LiquidJsonValue>) -> Self {
    Self {
      value: None,
      template,
      root_config: None,
    }
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

impl From<HashMap<String, Value>> for LiquidJsonConfig {
  fn from(value: HashMap<String, Value>) -> Self {
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

impl TryFrom<Value> for LiquidJsonConfig {
  type Error = Error;

  fn try_from(value: Value) -> Result<Self, Self::Error> {
    let value = match value {
      Value::Object(map) => map,
      _ => return Err(Error::ConfigurationTemplate("expected object".to_owned())),
    };

    let mut template = HashMap::new();
    for (k, v) in value {
      template.insert(k, v.into());
    }

    Ok(Self {
      value: None,
      template,
      root_config: None,
    })
  }
}
