use wick_config::WickConfiguration;
use wick_interface_types::ComponentSignature;
use wick_packet::{Entity, Invocation, PacketStream, RuntimeConfig};
pub use wick_runtime::error::RuntimeError;

use crate::error::HostError;
use crate::{AppHost, ComponentHost};

#[async_trait::async_trait]
pub trait Host {
  fn namespace(&self) -> &str;
  fn get_signature(&self, path: Option<&[&str]>, entity: Option<&Entity>) -> Result<ComponentSignature, HostError>;
  async fn invoke(&self, invocation: Invocation, data: Option<RuntimeConfig>) -> Result<PacketStream, HostError>;
  async fn invoke_deep(
    &self,
    path: Option<&[&str]>,
    invocation: Invocation,
    data: Option<RuntimeConfig>,
  ) -> Result<PacketStream, HostError>;
  fn get_active_config(&self) -> WickConfiguration;
}

#[derive(Debug)]
pub enum WickHost {
  App(AppHost),
  Component(ComponentHost),
}

#[async_trait::async_trait]
impl Host for WickHost {
  fn namespace(&self) -> &str {
    match self {
      Self::App(h) => h.namespace(),
      Self::Component(h) => h.namespace(),
    }
  }

  fn get_signature(&self, path: Option<&[&str]>, entity: Option<&Entity>) -> Result<ComponentSignature, HostError> {
    match self {
      Self::App(h) => h.get_signature(path, entity),
      Self::Component(h) => h.get_signature(path, entity),
    }
  }

  async fn invoke(&self, invocation: Invocation, data: Option<RuntimeConfig>) -> Result<PacketStream, HostError> {
    match self {
      Self::App(h) => h.invoke(invocation, data).await,
      Self::Component(h) => h.invoke(invocation, data).await,
    }
  }
  async fn invoke_deep(
    &self,
    path: Option<&[&str]>,
    invocation: Invocation,
    data: Option<RuntimeConfig>,
  ) -> Result<PacketStream, HostError> {
    match self {
      Self::App(h) => h.invoke_deep(path, invocation, data).await,
      Self::Component(h) => h.invoke_deep(path, invocation, data).await,
    }
  }

  fn get_active_config(&self) -> WickConfiguration {
    match self {
      Self::App(h) => h.get_active_config(),
      Self::Component(h) => h.get_active_config(),
    }
  }
}
