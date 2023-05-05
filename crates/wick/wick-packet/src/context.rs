use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Error;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct OperationConfig(HashMap<String, Value>);

impl OperationConfig {
  /// Get a value from the configuration map, deserializing it into the target type.
  pub fn get_into<T>(&self, key: &str) -> Result<T, Error>
  where
    T: DeserializeOwned,
  {
    let value = self.0.get(key).ok_or_else(|| Error::ContextKey(key.to_owned()))?;
    serde_json::from_value(value.clone()).map_err(|_| Error::ContextKey(key.to_owned()))
  }
  /// Get a value from the configuration map.
  #[must_use]
  pub fn get(&self, key: &str) -> Option<&Value> {
    self.0.get(key)
  }

  /// Get an iterator over the keys and values in the configuration.
  pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
    self.0.iter()
  }
}

impl From<HashMap<String, Value>> for OperationConfig {
  fn from(value: HashMap<String, Value>) -> Self {
    Self(value)
  }
}

impl From<OperationConfig> for HashMap<String, Value> {
  fn from(value: OperationConfig) -> Self {
    value.0
  }
}

impl TryFrom<Value> for OperationConfig {
  type Error = Error;

  fn try_from(value: Value) -> Result<Self, Self::Error> {
    match value {
      Value::Object(map) => {
        let mut hm = HashMap::new();
        for (k, v) in map {
          hm.insert(k, v);
        }
        Ok(Self(hm))
      }
      _ => Err(Error::BadJson),
    }
  }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct ContextTransport<T>
where
  T: std::fmt::Debug + Serialize,
{
  pub config: T,
  pub seed: Option<u64>,
}

impl<T> ContextTransport<T>
where
  T: std::fmt::Debug + Serialize,
{
  pub fn new(config: T, seed: Option<u64>) -> Self {
    Self { config, seed }
  }
}
