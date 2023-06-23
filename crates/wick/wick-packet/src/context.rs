use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ComponentReference, Error, InherentData};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RuntimeConfig(HashMap<String, Value>);

impl RuntimeConfig {
  /// Get a value from the configuration map, deserializing it into the target type.
  pub fn coerce_key<T>(&self, key: &str) -> Result<T, Error>
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
}

impl IntoIterator for RuntimeConfig {
  type Item = (String, Value);
  type IntoIter = std::collections::hash_map::IntoIter<String, Value>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

impl From<HashMap<String, Value>> for RuntimeConfig {
  fn from(value: HashMap<String, Value>) -> Self {
    Self(value)
  }
}

impl From<RuntimeConfig> for HashMap<String, Value> {
  fn from(value: RuntimeConfig) -> Self {
    value.0
  }
}

impl<K, const N: usize> From<[(K, Value); N]> for RuntimeConfig
where
  K: Into<String>,
{
  /// # Examples
  ///
  /// ```
  /// use wick_packet::RuntimeConfig;
  ///
  /// let config = RuntimeConfig::from([("key1", "value".into()), ("key2", 4.into())]);
  /// let key1 = config.coerce_key::<String>("key1").unwrap();
  /// assert_eq!(key1, "value");
  ///
  /// let key2 = config.coerce_key::<u16>("key2").unwrap();
  /// assert_eq!(key2, 4);
  ///
  /// ```
  fn from(arr: [(K, Value); N]) -> Self {
    Self(HashMap::from_iter(arr.into_iter().map(|(k, v)| (k.into(), v))))
  }
}

impl TryFrom<Value> for RuntimeConfig {
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
