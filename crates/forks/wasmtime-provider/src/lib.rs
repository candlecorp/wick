use std::error::Error;

use parking_lot::Mutex;
use wapc::{
  ModuleState,
  WapcFunctions,
  WasiParams,
  WebAssemblyEngineProvider,
  HOST_NAMESPACE,
};
use wasmtime::{
  AsContextMut,
  Engine,
  Extern,
  ExternType,
  Func,
  Instance,
  Linker,
  Module,
  Store,
};
use wasmtime_wasi::WasiCtx;

// namespace needed for some language support
const WASI_UNSTABLE_NAMESPACE: &str = "wasi_unstable";
const WASI_SNAPSHOT_PREVIEW1_NAMESPACE: &str = "wasi_snapshot_preview1";

use std::sync::{
  Arc,
  RwLock,
};

type HostType = Arc<Mutex<ModuleState>>;

#[macro_use]
extern crate log;

mod callbacks;
mod wasi;

struct EngineInner {
  instance: Arc<RwLock<Instance>>,
  guest_call_fn: Func,
  host: HostType,
}

struct WapcStore {
  wasi_ctx: WasiCtx,
}

/// A waPC engine provider that encapsulates the Wasmtime WebAssembly runtime
pub struct WasmtimeEngineProvider {
  inner: Option<EngineInner>,
  modbytes: Vec<u8>,
  store: Store<WapcStore>,
  engine: Engine,
  linker: Linker<WapcStore>,
}

impl WasmtimeEngineProvider {
  /// Creates a new instance of the wasmtime provider
  pub fn new(buf: &[u8], wasi: Option<WasiParams>) -> WasmtimeEngineProvider {
    let engine = Engine::default();
    let mut linker: Linker<WapcStore> = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| &mut s.wasi_ctx).unwrap();
    let wasi_default = WasiParams::default();
    let wasi_params = wasi.as_ref().unwrap_or(&wasi_default);
    let wasi_ctx = wasi::init_ctx(
      &wasi::compute_preopen_dirs(&wasi_params.preopened_dirs, &wasi_params.map_dirs).unwrap(),
      &wasi_params.argv,
      &wasi_params.env_vars,
    )
    .unwrap();
    let store = Store::new(&engine, WapcStore { wasi_ctx });
    WasmtimeEngineProvider {
      inner: None,
      modbytes: buf.to_vec(),
      store,
      engine,
      linker,
    }
  }
}

impl WebAssemblyEngineProvider for WasmtimeEngineProvider {
  fn init(&mut self, host: HostType) -> Result<(), Box<dyn Error>> {
    let instance = instance_from_buffer(
      &mut self.store,
      &self.engine,
      &self.modbytes,
      host.clone(),
      &self.linker,
    )?;
    let instance_ref = Arc::new(RwLock::new(instance));
    let gc = guest_call_fn(self.store.as_context_mut(), instance_ref.clone())?;
    self.inner = Some(EngineInner {
      instance: instance_ref,
      guest_call_fn: gc,
      host,
    });
    self.initialize()?;
    Ok(())
  }

  fn call(&mut self, op_length: i32, msg_length: i32) -> Result<i32, Box<dyn Error>> {
    let engine_inner = self.inner.as_ref().unwrap();
    let call = engine_inner
      .guest_call_fn
      .call(&mut self.store, &[op_length.into(), msg_length.into()]);

    match call {
      Ok(result) => {
        let result: i32 = result[0].i32().unwrap();
        Ok(result)
      }
      Err(e) => {
        error!("Failure invoking guest module handler: {:?}", e);
        engine_inner.host.lock().set_guest_error(e.to_string());
        Ok(0)
      }
    }
  }

  fn replace(&mut self, module: &[u8]) -> Result<(), Box<dyn Error>> {
    info!(
      "HOT SWAP - Replacing existing WebAssembly module with new buffer, {} bytes",
      module.len()
    );

    let new_instance = instance_from_buffer(
      &mut self.store,
      &self.engine,
      module,
      self.inner.as_ref().unwrap().host.clone(),
      &self.linker,
    )?;
    *self.inner.as_ref().unwrap().instance.write().unwrap() = new_instance;

    self.initialize()
  }
}

impl WasmtimeEngineProvider {
  fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
    for starter in wapc::WapcFunctions::REQUIRED_STARTS.iter() {
      if let Some(ext) = self
        .inner
        .as_ref()
        .unwrap()
        .instance
        .read()
        .unwrap()
        .get_export(&mut self.store, starter)
      {
        ext.into_func().unwrap().call(&mut self.store, &[])?;
      }
    }
    Ok(())
  }
}

fn instance_from_buffer(
  store: &mut Store<WapcStore>,
  engine: &Engine,
  buf: &[u8],
  state: HostType,
  linker: &Linker<WapcStore>,
) -> Result<Instance, Box<dyn Error>> {
  let module = Module::new(engine, buf).unwrap();
  let imports = arrange_imports(&module, state, store, linker);
  Ok(wasmtime::Instance::new(store.as_context_mut(), &module, imports?.as_slice()).unwrap())
}

/// wasmtime requires that the list of callbacks be "zippable" with the list
/// of module imports. In order to ensure that both lists are in the same
/// order, we have to loop through the module imports and instantiate the
/// corresponding callback. We **cannot** rely on a predictable import order
/// in the wasm module
#[allow(clippy::unnecessary_wraps)]
fn arrange_imports(
  module: &Module,
  host: HostType,
  store: &mut impl AsContextMut<Data = WapcStore>,
  linker: &Linker<WapcStore>,
) -> Result<Vec<Extern>, Box<dyn Error>> {
  Ok(
    module
      .imports()
      .filter_map(|imp| {
        if let ExternType::Func(_) = imp.ty() {
          match imp.module() {
            HOST_NAMESPACE => Some(callback_for_import(
              store.as_context_mut(),
              imp.name()?,
              host.clone(),
            )),
            WASI_SNAPSHOT_PREVIEW1_NAMESPACE | WASI_UNSTABLE_NAMESPACE => {
              linker.get_by_import(store.as_context_mut(), &imp)
            }
            other => panic!("import module `{}` was not found", other), //TODO: get rid of panic
          }
        } else {
          None
        }
      })
      .collect(),
  )
}

fn callback_for_import(store: impl AsContextMut, import: &str, host: HostType) -> Extern {
  match import {
    WapcFunctions::HOST_CONSOLE_LOG => callbacks::console_log_func(store, host).into(),
    WapcFunctions::HOST_CALL => callbacks::host_call_func(store, host).into(),
    WapcFunctions::GUEST_REQUEST_FN => callbacks::guest_request_func(store, host).into(),
    WapcFunctions::HOST_RESPONSE_FN => callbacks::host_response_func(store, host).into(),
    WapcFunctions::HOST_RESPONSE_LEN_FN => callbacks::host_response_len_func(store, host).into(),
    WapcFunctions::GUEST_RESPONSE_FN => callbacks::guest_response_func(store, host).into(),
    WapcFunctions::GUEST_ERROR_FN => callbacks::guest_error_func(store, host).into(),
    WapcFunctions::HOST_ERROR_FN => callbacks::host_error_func(store, host).into(),
    WapcFunctions::HOST_ERROR_LEN_FN => callbacks::host_error_len_func(store, host).into(),
    _ => unreachable!(),
  }
}

// Called once, then the result is cached. This returns a `Func` that corresponds
// to the `__guest_call` export
fn guest_call_fn(
  store: impl AsContextMut,
  instance: Arc<RwLock<Instance>>,
) -> Result<Func, Box<dyn Error>> {
  if let Some(func) = instance
    .read()
    .unwrap()
    .get_func(store, WapcFunctions::GUEST_CALL)
  {
    Ok(func)
  } else {
    Err("Guest module did not export __guest_call function!".into())
  }
}
