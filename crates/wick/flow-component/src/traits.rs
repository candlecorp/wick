use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;
use wick_interface_types::{ComponentSignature, OperationSignature};
use wick_packet::{ComponentReference, InherentData, Invocation, PacketStream, RuntimeConfig};

use crate::context::Context;
use crate::{BoxFuture, ComponentError};

/// A [Component] that can be easily cloned and shared.
pub type SharedComponent = Arc<dyn Component + Send + Sync>;

/// The [Component] trait allows you to build components that operation within a `flow-graph-interpreter`
pub trait Component {
  /// The `handle` method is called when a component's operation is invoked. The component is expected to delegate to the appopriate operation based on the [Invocation] target.
  fn handle(
    &self,
    invocation: Invocation,
    data: Option<RuntimeConfig>,
    callback: Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>>;

  /// The `signature` method returns the [ComponentSignature] for the component.
  fn signature(&self) -> &ComponentSignature;

  /// The `shutdown` method is called when the component is shutdown.
  fn shutdown(&self) -> BoxFuture<Result<(), ComponentError>> {
    // Override if you need a more explicit shutdown.
    Box::pin(async move { Ok(()) })
  }
}

/// The [RenderConfiguration] trait allows you to build structs that can decode and render dynamic configuration.
pub trait RenderConfiguration {
  /// The configuration type for the implementer.
  type Config: std::fmt::Debug + DeserializeOwned + Serialize + Send + Sync + 'static;

  /// The configuration source for the implementer.
  type ConfigSource: std::fmt::Debug;

  /// The `decode_config` function decodes a [RuntimeConfig] into the implementer's configuration type.
  fn decode_config(data: Option<Self::ConfigSource>) -> Result<Self::Config, ComponentError>;
}

/// The [Operation] trait allows you to build operations that can be invoked by a [Component].
pub trait Operation {
  /// The static identifier for the operation.
  const ID: &'static str;

  /// The configuration type for the operation.
  type Config: std::fmt::Debug + DeserializeOwned + Serialize + Send + Sync + 'static;

  /// The `handle` method is called when the operation is invoked.
  fn handle(
    &self,
    invocation: Invocation,
    context: Context<Self::Config>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>>;

  /// The `signature` method returns the [OperationSignature] for the operation.
  fn get_signature(&self, config: Option<&Self::Config>) -> &OperationSignature;

  /// The `input_names` method returns the names of the inputs for the operation.
  fn input_names(&self, config: &Self::Config) -> Vec<String>;
}

/// The [RuntimeCallback] type is used to invoke other components within the executing runtime.
pub type RuntimeCallback = dyn Fn(
    ComponentReference,
    String,
    PacketStream,
    InherentData,
    Option<RuntimeConfig>,
    &tracing::Span,
  ) -> BoxFuture<'static, Result<PacketStream, ComponentError>>
  + Send
  + Sync;

#[must_use]
/// A [RuntimeCallback] that panics when invoked.
pub fn panic_callback() -> Arc<RuntimeCallback> {
  Arc::new(|_, _, _, _, _, _| {
    Box::pin(async move {
      panic!("Panic callback invoked. This should never happen outside of tests.");
    })
  })
}
