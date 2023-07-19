use std::collections::HashMap;
use std::str::FromStr;

use liquid_json::LiquidJsonValue;
use serde_json::Value;
use tracing::debug_span;
use wick_packet::RuntimeConfig;

use crate::config::LiquidJsonConfig;
use crate::error::ManifestError;

#[derive(Debug, Clone, PartialEq, property::Property, serde::Serialize)]
/// A liquid template configuration that retains portions of its context
/// and can be unrendered into the original template or value.
pub struct TemplateConfig<V>
where
  V: Clone + std::fmt::Debug + std::fmt::Display + PartialEq + FromStr,
{
  #[property(skip)]
  #[serde(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) value: Option<V>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) template: Option<LiquidJsonValue>,
  #[serde(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) root_config: Option<RuntimeConfig>,
}

impl<T> Default for TemplateConfig<T>
where
  T: Clone + std::fmt::Debug + std::fmt::Display + PartialEq + FromStr + Default,
{
  fn default() -> Self {
    Self {
      value: Default::default(),
      template: Default::default(),
      root_config: Default::default(),
    }
  }
}

impl<T> std::fmt::Display for TemplateConfig<T>
where
  T: Clone + std::fmt::Debug + std::fmt::Display + PartialEq + FromStr,
  T: std::fmt::Display,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match (&self.value, &self.template) {
      (Some(v), _) => write!(f, "{}", v),
      (None, Some(v)) => write!(f, "{}", v.as_json()),
      (None, None) => write!(f, "invalid resource, no value and no template"),
    }
  }
}

impl<T> TemplateConfig<T>
where
  T: Clone + std::fmt::Debug + std::fmt::Display + PartialEq + FromStr,
{
  #[must_use]
  /// Create a new [TemplateConfig] from a value.
  pub fn new_value(value: T) -> Self {
    Self {
      value: Some(value),
      template: None,
      root_config: None,
    }
  }

  #[must_use]
  /// Create a new [TemplateConfig] from a template string.
  pub fn new_template(value: String) -> Self {
    Self {
      value: None,
      template: Some(LiquidJsonValue::new(Value::String(value))),
      root_config: None,
    }
  }

  /// Retrieve a previously rendered value rendered.
  pub fn value(&self) -> Option<&T> {
    self.value.as_ref()
  }

  /// Set the value rendered held by this configuration to be cached.
  pub fn set_value(&mut self, value: T) {
    self.value = Some(value);
  }

  #[must_use]
  /// Retrieve the previously rendered value or panic.
  ///
  /// This should only be used passed the boundary of configuration rendering.
  pub fn value_unchecked(&self) -> &T {
    self.value.as_ref().unwrap()
  }

  /// Return a [TemplateConfig] back to either its template or, if a template, does not exist, its value.
  pub fn unrender(&self) -> Result<String, ManifestError> {
    self.template.as_ref().map_or_else(
      || {
        self.value.as_ref().map_or_else(
          || Err(ManifestError::UnrenderedConfiguration(format!("{:?}", self.template))),
          |value| Ok((*value).to_string()),
        )
      },
      |template| value_to_string(template.as_json()),
    )
  }

  /// Render a [TemplateConfig] into the desired value, creating a context from the passed configuration.
  pub fn render(&self, root: Option<&RuntimeConfig>, env: Option<&HashMap<String, String>>) -> Result<T, crate::Error> {
    if let Some(value) = &self.value {
      return Ok(value.clone());
    }
    let ctx =
      debug_span!("template-config").in_scope(|| LiquidJsonConfig::make_context(None, root, None, env, None))?;

    if let Some(template) = &self.template {
      let rendered = template
        .render(&ctx)
        .map_err(|e| crate::Error::ConfigurationTemplate(e.to_string()))?;
      let rendered = value_to_string(&rendered)?;

      Ok(rendered.parse::<T>().map_err(|_| {
        crate::Error::ConfigurationTemplate(format!(
          "could not convert {} into {}",
          rendered,
          std::any::type_name::<T>()
        ))
      })?)
    } else {
      Err(crate::Error::ConfigurationTemplate(
        "No value or template specified".to_owned(),
      ))
    }
  }
}

fn value_to_string(value: &Value) -> Result<String, ManifestError> {
  match value {
    serde_json::Value::String(v) => Ok(v.clone()),
    serde_json::Value::Number(v) => Ok(v.to_string()),
    serde_json::Value::Null => Ok("".to_owned()),
    serde_json::Value::Bool(v) => Ok(v.to_string()),
    serde_json::Value::Array(_) => Err(ManifestError::TemplateStructure),
    serde_json::Value::Object(_) => Err(ManifestError::TemplateStructure),
  }
}
