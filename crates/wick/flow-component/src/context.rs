use std::sync::Arc;

use seeded_random::{Random, Seed};
use serde::de::DeserializeOwned;
use serde::Serialize;
use wick_packet::{date_from_millis, ContextTransport, DateTime, InherentData};

#[cfg(target_family = "wasm")]
/// A conditional trait that implements Send if the target is not wasm.
pub trait LocalAwareSend {}
#[cfg(not(target_family = "wasm"))]
/// A conditional trait that implements Send if the target is not wasm.
pub trait LocalAwareSend: Send {}

#[cfg(target_family = "wasm")]
impl<T> LocalAwareSend for T {}

#[cfg(not(target_family = "wasm"))]
impl<T> LocalAwareSend for T where T: Send {}

#[derive(Clone)]
/// A context that is passed to a component's operations.
pub struct Context<T>
where
  T: std::fmt::Debug,
  T: LocalAwareSend,
{
  /// Operation-specific configuration.
  pub config: Arc<T>,
  /// Inherent data passed to the operation.
  pub inherent: InherentContext,
  #[cfg(feature = "invocation")]
  /// A callback to invoke other components within the executing runtime.
  pub callback: Arc<crate::RuntimeCallback>,
}

impl<T> std::fmt::Debug for Context<T>
where
  T: std::fmt::Debug + DeserializeOwned + Serialize,
  T: LocalAwareSend,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Context").field("config", &self.config).finish()
  }
}

impl<T> From<ContextTransport<T>> for Context<T>
where
  T: std::fmt::Debug + Serialize + DeserializeOwned,
  T: LocalAwareSend,
{
  fn from(value: ContextTransport<T>) -> Self {
    Self {
      inherent: InherentContext {
        rng: Random::from_seed(Seed::unsafe_new(value.inherent.seed)),
        timestamp: date_from_millis(value.inherent.timestamp).unwrap(),
      },
      config: Arc::new(value.config),
      #[cfg(feature = "invocation")]
      callback: crate::panic_callback(),
    }
  }
}

#[derive(Debug)]
/// Inherent data passed to an operation.
pub struct InherentContext {
  /// A random number generator initialized from the invocation seed.
  pub rng: Random,
  /// The timestamp of the invocation.
  pub timestamp: DateTime,
}

impl Clone for InherentContext {
  fn clone(&self) -> Self {
    Self {
      rng: Random::from_seed(self.rng.seed()),
      timestamp: self.timestamp,
    }
  }
}

impl From<InherentContext> for InherentData {
  fn from(value: InherentContext) -> Self {
    Self {
      seed: value.rng.gen(),
      timestamp: value.timestamp.timestamp_millis() as _,
    }
  }
}

impl From<InherentData> for InherentContext {
  fn from(value: InherentData) -> Self {
    Self {
      rng: Random::from_seed(Seed::unsafe_new(value.seed)),
      timestamp: date_from_millis(value.timestamp).unwrap(),
    }
  }
}

impl<T> Context<T>
where
  T: std::fmt::Debug,
  T: LocalAwareSend,
{
  /// Create a new context.
  #[cfg(feature = "invocation")]
  pub fn new(config: T, inherent: &InherentData, callback: Arc<crate::RuntimeCallback>) -> Self {
    Self {
      inherent: InherentContext {
        rng: Random::from_seed(Seed::unsafe_new(inherent.seed)),
        timestamp: date_from_millis(inherent.timestamp).unwrap(),
      },
      config: Arc::new(config),
      callback,
    }
  }

  /// Create a new context.
  #[cfg(not(feature = "invocation"))]
  pub fn new(config: T, inherent: &InherentData) -> Self {
    Self {
      inherent: InherentContext {
        rng: Random::from_seed(Seed::unsafe_new(inherent.seed)),
        timestamp: date_from_millis(inherent.timestamp).unwrap(),
      },
      config: Arc::new(config),
    }
  }
}
