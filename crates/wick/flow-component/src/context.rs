use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;
use wick_packet::{ContextTransport, InherentData};

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
  pub inherent: Option<InherentData>,
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
      inherent: value.inherent,
      config: Arc::new(value.config),
      #[cfg(feature = "invocation")]
      callback: crate::panic_callback(),
    }
  }
}

impl<T> Context<T>
where
  T: std::fmt::Debug,
  T: LocalAwareSend,
{
  #[cfg(feature = "invocation")]
  /// Create a new context.
  pub fn new(config: T, inherent: Option<InherentData>, callback: Arc<crate::RuntimeCallback>) -> Self {
    Self {
      inherent,
      config: Arc::new(config),
      callback,
    }
  }

  #[cfg(not(feature = "invocation"))]
  /// Create a new context.
  pub fn new(config: T, inherent: Option<InherentData>) -> Self {
    Self {
      inherent,
      config: Arc::new(config),
    }
  }
}
