use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;
use wick_packet::ContextTransport;

#[derive(Clone)]
pub struct Context<T>
where
  T: std::fmt::Debug,
{
  pub config: Arc<T>,
  pub seed: Option<u64>,
  #[cfg(feature = "traits")]
  pub callback: Arc<crate::RuntimeCallback>,
}

impl<T> std::fmt::Debug for Context<T>
where
  T: std::fmt::Debug + DeserializeOwned + Serialize,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Context").field("config", &self.config).finish()
  }
}

impl<T> From<ContextTransport<T>> for Context<T>
where
  T: std::fmt::Debug + Serialize + DeserializeOwned,
{
  fn from(value: ContextTransport<T>) -> Self {
    Self {
      seed: value.seed,
      config: Arc::new(value.config),
      #[cfg(feature = "traits")]
      callback: crate::panic_callback(),
    }
  }
}

impl<T> Context<T>
where
  T: std::fmt::Debug,
{
  #[cfg(feature = "traits")]
  pub fn new(config: T, seed: Option<u64>, callback: Arc<crate::RuntimeCallback>) -> Self {
    Self {
      seed,
      config: Arc::new(config),
      callback,
    }
  }

  #[cfg(not(feature = "traits"))]
  pub fn new(config: T, seed: Option<u64>) -> Self {
    Self {
      seed,
      config: Arc::new(config),
    }
  }
}
