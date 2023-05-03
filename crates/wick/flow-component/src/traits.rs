use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;
use wick_interface_types::{ComponentSignature, OperationSignature};
use wick_packet::{ComponentReference, InherentData, Invocation, OperationConfig, PacketStream};

use crate::context::Context;
use crate::{BoxFuture, ComponentError};

pub type SharedComponent = Arc<dyn Component + Send + Sync>;

pub trait Component {
  fn handle(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    data: Option<OperationConfig>,
    callback: Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>>;
  fn list(&self) -> &ComponentSignature;
  fn init(&self) -> BoxFuture<Result<(), ComponentError>> {
    // Override if you need a more explicit init.
    Box::pin(async move { Ok(()) })
  }

  fn shutdown(&self) -> BoxFuture<Result<(), ComponentError>> {
    // Override if you need a more explicit shutdown.
    Box::pin(async move { Ok(()) })
  }
}
pub trait Operation {
  const ID: &'static str;
  type Config: std::fmt::Debug + DeserializeOwned + Serialize + Send + Sync + 'static;
  fn handle(
    &self,
    payload: PacketStream,
    context: Context<Self::Config>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>>;

  fn get_signature(&self, config: Option<&Self::Config>) -> &OperationSignature;

  fn input_names(&self, config: &Self::Config) -> Vec<String>;

  fn decode_config(data: Option<OperationConfig>) -> Result<Self::Config, ComponentError>;
}

pub type RuntimeCallback = dyn Fn(
    ComponentReference,
    String,
    PacketStream,
    Option<InherentData>,
    Option<OperationConfig>,
  ) -> BoxFuture<'static, Result<PacketStream, ComponentError>>
  + Send
  + Sync;

#[must_use]
pub fn panic_callback() -> Arc<RuntimeCallback> {
  Arc::new(|_, _, _, _, _| {
    Box::pin(async move {
      panic!("Panic callback invoked. This should never happen outside of tests.");
    })
  })
}
