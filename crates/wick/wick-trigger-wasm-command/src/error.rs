#[derive(thiserror::Error, Debug)]
#[allow(clippy::exhaustive_enums)]
/// Crate error.
pub enum Error {
  /// Failed to load component
  #[error("failed load component, {0}")]
  ComponentLoad(wasmtime::Error),

  /// Failed to fetch wasm bytes
  #[error("failed to fetch wasm bytes, {0}")]
  ComponentFetch(Box<dyn std::error::Error + Send + Sync + 'static>),

  /// Failed to link component
  #[error("failed to link component, {0}")]
  Linker(wasmtime::Error),

  /// Failed to build WASI context
  #[error("failed to build WASI context, {0}")]
  WasiBuild(wasmtime::Error),

  /// Could not instantiate component.
  #[error("could not instantiate component, {0}")]
  Instantiation(wasmtime::Error),

  /// Could not link with WASI Command bindings.
  #[error("could not link with WASI Command bindings, {0}")]
  WasiCommand(wasmtime::Error),

  /// Error running component.
  #[error("component returned with error, {0}")]
  CommandRun(wasmtime::Error),
}

impl From<Error> for wick_trigger::Error {
  fn from(value: Error) -> Self {
    wick_trigger::Error::new_context("wasm-command", wick_trigger::ErrorKind::Trigger(Box::new(value)))
  }
}
