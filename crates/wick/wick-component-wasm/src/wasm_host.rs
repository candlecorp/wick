use std::sync::Arc;
use std::time::Instant;

use parking_lot::Mutex;
use wasmrs::RSocket;
use wasmrs_host::{Host, WasiParams};
use wick_interface_types::ComponentSignature;
use wick_packet::{from_wasmrs, into_wasmrs, PacketStream};
use wick_wascap::{Claims, CollectionClaims};

use crate::collection::HostLinkCallback;
use crate::error::WasmCollectionError;
use crate::wasm_module::WickWasmModule;
use crate::{Error, Result};

#[must_use]
pub struct WasmHostBuilder {
  wasi_params: Option<WasiParams>,
  callback: Option<Box<HostLinkCallback>>,
  min_threads: usize,
  max_threads: usize,
}

impl std::fmt::Debug for WasmHostBuilder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("WasmHostBuilder")
      .field("wasi_params", &self.wasi_params)
      .finish()
  }
}

impl WasmHostBuilder {
  pub fn new() -> Self {
    Self {
      wasi_params: None,
      callback: None,
      min_threads: 1,
      max_threads: 1,
    }
  }

  pub fn wasi_params(mut self, params: WasiParams) -> Self {
    self.wasi_params = Some(params);
    self
  }

  pub fn link_callback(mut self, callback: Box<HostLinkCallback>) -> Self {
    self.callback = Some(callback);
    self
  }

  pub fn preopened_dirs(mut self, dirs: Vec<String>) -> Self {
    let mut params = self.wasi_params.take().unwrap_or_default();
    params.preopened_dirs = dirs;
    self.wasi_params.replace(params);
    self
  }

  pub fn build(self, module: &WickWasmModule) -> Result<WasmHost> {
    WasmHost::try_load(
      module,
      self.wasi_params,
      &self.callback,
      self.min_threads,
      self.max_threads,
    )
  }

  pub fn max_threads(mut self, max_threads: usize) -> Self {
    self.max_threads = max_threads;
    self
  }

  pub fn min_threads(mut self, min_threads: usize) -> Self {
    self.min_threads = min_threads;
    self
  }
}

impl Default for WasmHostBuilder {
  fn default() -> Self {
    Self::new()
  }
}

#[derive()]
pub struct WasmHost {
  host: Arc<Mutex<Host>>,
  claims: Claims<CollectionClaims>,
  _rng: seeded_random::Random,
}

impl std::fmt::Debug for WasmHost {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("WasmHost").field("claims", &self.claims).finish()
  }
}

impl WasmHost {
  pub fn try_load(
    module: &WickWasmModule,
    wasi_options: Option<WasiParams>,
    _callback: &Option<Box<HostLinkCallback>>,
    _min_threads: usize,
    _max_threads: usize,
  ) -> Result<Self> {
    let jwt = &module.token.jwt;

    wick_wascap::validate_token::<CollectionClaims>(jwt).map_err(|e| Error::ClaimsInvalid(e.to_string()))?;

    let time = Instant::now();

    let engine = wasmrs_wasmtime::WasmtimeBuilder::new(&module.bytes).enable_cache(None);
    let engine = if let Some(wasi_options) = wasi_options {
      engine.wasi_params(wasi_options)
    } else {
      engine
    };
    let engine = engine
      .build()
      .map_err(|e| WasmCollectionError::EngineFailure(e.to_string()))?;
    trace!(duration_μs = %time.elapsed().as_micros(), "wasmtime instance loaded");

    let host = Host::new(engine).map_err(|e| WasmCollectionError::EngineFailure(e.to_string()))?;

    debug!(duration_μs = ?time.elapsed().as_micros(), "wasmtime initialize");

    Ok(Self {
      claims: module.claims().clone(),
      host: Arc::new(Mutex::new(host)),
      _rng: seeded_random::Random::new(),
    })
  }

  pub fn call(&self, component_name: &str, stream: PacketStream, _config: Option<&[u8]>) -> Result<PacketStream> {
    debug!(component = component_name, "wasm invoke");

    let now = Instant::now();
    let ctx = self.host.lock().new_context(128 * 1024, 128 * 1024).unwrap();
    let index = ctx
      .get_export("wick", component_name)
      .map_err(|e| crate::Error::ComponentNotFound(e.to_string(), ctx.get_exports()))?;
    let s = into_wasmrs(index, stream);
    let out = ctx.request_channel(Box::new(s));
    trace!(
      component = component_name,
      duration_μs = ?now.elapsed().as_micros(),
      "wasm call finished"
    );
    Ok(from_wasmrs(out))
  }

  pub fn get_operations(&self) -> &ComponentSignature {
    let claims = &self.claims;
    &claims.metadata.as_ref().unwrap().interface
  }
}
