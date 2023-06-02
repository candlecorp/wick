use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;
use wick_packet::{ContextTransport, InherentData};

#[cfg(target_family = "wasm")]
pub trait LocalAwareSend {}
#[cfg(not(target_family = "wasm"))]
pub trait LocalAwareSend: Send {}

#[cfg(target_family = "wasm")]
impl<T> LocalAwareSend for T {}

#[cfg(not(target_family = "wasm"))]
impl<T> LocalAwareSend for T where T: Send {}

#[derive(Clone)]
pub struct Context<T>
where
  T: std::fmt::Debug,
  T: LocalAwareSend,
{
  pub config: Arc<T>,
  pub inherent: Option<InherentData>,
  #[cfg(feature = "invocation")]
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
  pub fn new(config: T, inherent: Option<InherentData>, callback: Arc<crate::RuntimeCallback>) -> Self {
    Self {
      inherent,
      config: Arc::new(config),
      callback,
    }
  }

  #[cfg(not(feature = "invocation"))]
  pub fn new(config: T, inherent: Option<InherentData>) -> Self {
    Self {
      inherent,
      config: Arc::new(config),
    }
  }
}

pub trait ProviderContext {
  type Provided;
  fn provided(&self) -> Self::Provided;
}
