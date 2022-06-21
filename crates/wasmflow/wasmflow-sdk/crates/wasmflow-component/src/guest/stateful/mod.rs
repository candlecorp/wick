pub mod error;

#[cfg(target_arch = "wasm32")]
pub mod wasm {
  use crate::guest::wasm::BoxedFuture;
  use crate::guest::BoxedError;
  pub trait Dispatcher {
    type Context;

    /// Dispatch method
    fn dispatch(
      &self,
      op: &'static str,
      payload: &'static [u8],
      context: Self::Context,
    ) -> BoxedFuture<Result<Vec<u8>, BoxedError>>;
  }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod native {

  use wasmflow_streams::PacketStream;

  use crate::guest::native::BoxedFuture;
  use crate::guest::BoxedError;

  pub trait Dispatcher {
    type Context;

    fn dispatch(
      &self,
      payload: wasmflow_invocation::Invocation,
      context: Self::Context,
    ) -> BoxedFuture<Result<PacketStream, BoxedError>>;
  }
}

/// A trait for ephemeral components that take inputs batched together for a single run.
pub trait BatchedJobExecutor {
  /// The type of the main payload for the component.
  type Payload: std::fmt::Debug;
  /// The type of the configuration object passed to the component.
  type Config: std::fmt::Debug;
  /// The type of the Context object passed to a component job.
  type Context: std::fmt::Debug;
  /// The type of the state object passed to and returned from a component.
  type State: std::fmt::Debug;
  /// The return type of the component.
  type Return: Send + Sync;

  /// [BatchedJobExecutor::execute] for WASM components that do not require a future be Send + Sync.
  #[cfg(target_arch = "wasm32")]
  fn execute(
    &self,
    payload: wasmflow_boundary::IncomingPayload<Self::Payload, Self::Config, Self::State>,
    context: Self::Context,
  ) -> super::wasm::BoxedFuture<Result<Self::Return, Box<dyn std::error::Error + Send + Sync>>>;

  /// [BatchedJobExecutor::execute] that kicks off a run of a component, passing along an [wasmflow_boundary::IncomingPayload] and a persistent context.
  #[cfg(not(target_arch = "wasm32"))]
  fn execute(
    &self,
    payload: wasmflow_boundary::IncomingPayload<Self::Payload, Self::Config, Self::State>,
    context: Self::Context,
  ) -> super::native::BoxedFuture<Result<Self::Return, Box<dyn std::error::Error + Send + Sync>>>;
}
