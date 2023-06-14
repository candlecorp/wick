use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ComponentReference, Error, InherentData};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GenericConfig(HashMap<String, Value>);

impl GenericConfig {
  /// Get a value from the configuration map, deserializing it into the target type.
  pub fn get_into<T>(&self, key: &str) -> Result<T, Error>
  where
    T: DeserializeOwned,
  {
    let value = self.0.get(key).ok_or_else(|| Error::ContextKey(key.to_owned()))?;
    serde_json::from_value(value.clone()).map_err(|_| Error::ContextKey(key.to_owned()))
  }

  /// Check if a value exists in the configuration map.
  #[must_use]
  pub fn has(&self, key: &str) -> bool {
    self.0.contains_key(key)
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

  /// Get an iterator over the owned keys and values in the configuration.
  pub fn into_iter(self) -> impl Iterator<Item = (String, Value)> {
    self.0.into_iter()
  }
}

impl From<HashMap<String, Value>> for GenericConfig {
  fn from(value: HashMap<String, Value>) -> Self {
    Self(value)
  }
}

impl From<GenericConfig> for HashMap<String, Value> {
  fn from(value: GenericConfig) -> Self {
    value.0
  }
}

impl TryFrom<Value> for GenericConfig {
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

#[derive(Deserialize, Serialize, Debug)]
pub struct ContextTransport<T>
where
  T: std::fmt::Debug + Serialize,
{
  pub config: T,
  pub inherent: InherentData,
  pub invocation: Option<InvocationRequest>,
}

impl<T> ContextTransport<T>
where
  T: std::fmt::Debug + Serialize,
{
  pub fn new(config: T, inherent: InherentData) -> Self {
    Self {
      config,
      inherent,
      invocation: None,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
pub struct InvocationRequest {
  pub reference: ComponentReference,
  pub operation: String,
}
