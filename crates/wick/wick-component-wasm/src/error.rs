#[derive(thiserror::Error, Debug)]
#[allow(clippy::exhaustive_enums)]
/// Crate error.
pub enum Error {
  /// Failed to initialize WASI context
  #[error("failed to initialize WASI context, {0}")]
  WasiCtx(wasmtime::Error),

  /// Failed to open directory for component
  #[error("failed to open directory for component, {0}")]
  OpenDir(std::io::Error),

  /// Failed to fetch wasm bytes
  #[error("failed to fetch wasm bytes, {0}")]
  ComponentFetch(wasmtime::Error),

  /// Failure dealing with asset reference
  #[error("failed to get asset path, {0}")]
  Asset(Box<dyn std::error::Error + Send + Sync>),

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
