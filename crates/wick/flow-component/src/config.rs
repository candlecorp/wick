use std::collections::HashMap;

use liquid_json::LiquidJsonValue;
use wick_packet::GenericConfig;

/// A generic configuration that may include LiquidJson values
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LiquidConfig(pub HashMap<String, LiquidJsonValue>);

impl LiquidConfig {
  /// Render a [LiquidConfig] into a [GenericConfig] with the passed context.
  pub fn render(&self, context: &serde_json::Value) -> Result<GenericConfig, liquid_json::Error> {
    let mut map = HashMap::new();
    for (k, v) in self.0.iter() {
      map.insert(k.clone(), v.render(context)?);
    }
    Ok(GenericConfig::from(map))
  }
}

impl From<HashMap<String, LiquidJsonValue>> for LiquidConfig {
  fn from(value: HashMap<String, LiquidJsonValue>) -> Self {
    Self(value)
  }
}

impl From<HashMap<String, serde_json::Value>> for LiquidConfig {
  fn from(value: HashMap<String, serde_json::Value>) -> Self {
    Self(value.into_iter().map(|(k, v)| (k, v.into())).collect())
  }
}

impl From<LiquidConfig> for HashMap<String, LiquidJsonValue> {
  fn from(value: LiquidConfig) -> Self {
    value.0
  }
}
