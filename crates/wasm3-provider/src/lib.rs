use std::cell::RefCell;
use std::error::Error;
use std::sync::Arc;

use wapc::{
  ModuleState,
  WapcFunctions,
  WebAssemblyEngineProvider,
  HOST_NAMESPACE,
};
use wasm3::{
  CallContext,
  Environment,
  Module,
  Runtime,
};

#[macro_use]
extern crate log;

mod callbacks;

const WASI_UNSTABLE: &str = "wasi_unstable";

pub struct Wasm3EngineProvider {
  inner: Option<InnerProvider>,
  modbytes: Vec<u8>,
}

impl Wasm3EngineProvider {
  pub fn new(buf: &[u8]) -> Wasm3EngineProvider {
    Wasm3EngineProvider {
      inner: None,
      modbytes: buf.to_vec(),
    }
  }
}

struct InnerProvider {
  rt: Runtime,
  mod_name: String,
}

impl WebAssemblyEngineProvider for Wasm3EngineProvider {
  fn init(&mut self, host: Arc<RefCell<ModuleState>>) -> Result<(), Box<dyn Error>> {
    info!("Initializing Wasm3 Engine");
    let env = Environment::new().expect("Unable to create environment");
    let rt = env.create_runtime(1024 * 120)?;
    let module = Module::parse(&env, &self.modbytes).map_err(|e| Box::new(e))?;

    let mut module = rt.load_module(module).map_err(|e| Box::new(e))?;

    let mod_name = module.name().to_string();

    let h = host.clone();
    if let Err(_e) = module.link_closure(
      HOST_NAMESPACE,
      WapcFunctions::HOST_CALL,
      move |ctx: &CallContext,
            (bd_ptr, bd_len, ns_ptr, ns_len, op_ptr, op_len, ptr, len): (
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
      )|
            -> i32 {
        callbacks::host_call(
          ctx,
          bd_ptr,
          bd_len,
          ns_ptr,
          ns_len,
          op_ptr,
          op_len,
          ptr,
          len,
          h.clone(),
        )
      },
    ) {
      warn!("Guest module did not import __host_call - functionality may be limited");
    }

    let h = host.clone();
    if let Err(_e) = module.link_closure(
      HOST_NAMESPACE,
      WapcFunctions::GUEST_REQUEST_FN,
      move |ctx: &CallContext, (op_ptr, ptr): (i32, i32)| {
        callbacks::guest_request(ctx, op_ptr, ptr, h.clone());
      },
    ) {
      error!("Module did not import __guest_request - will not work with waPC");
      return Err("Module did not import __guest_request - will not work with waPC".into());
    }

    let h = host.clone();
    if let Err(_e) = module.link_closure(
      HOST_NAMESPACE,
      WapcFunctions::HOST_CONSOLE_LOG,
      move |ctx: &CallContext, (ptr, len): (i32, i32)| {
        callbacks::console_log(ctx, ptr, len, h.clone())
      },
    ) {
      warn!("Module did not import __console_log");
    }

    let h = host.clone();
    if let Err(_e) = module.link_closure(
      HOST_NAMESPACE,
      WapcFunctions::HOST_RESPONSE_FN,
      move |ctx: &CallContext, ptr: i32| callbacks::host_response(ctx, ptr, h.clone()),
    ) {
      warn!("Module did not import __host_response");
    }

    let h = host.clone();
    if let Err(_e) = module.link_closure(
      HOST_NAMESPACE,
      WapcFunctions::HOST_RESPONSE_LEN_FN,
      move |ctx: &CallContext, ()| -> i32 { callbacks::host_response_length(ctx, h.clone()) },
    ) {
      warn!("Module did not import __host_response_len");
    }

    let h = host.clone();
    if let Err(_e) = module.link_closure(
      HOST_NAMESPACE,
      WapcFunctions::GUEST_RESPONSE_FN,
      move |ctx: &CallContext, (ptr, len): (i32, i32)| {
        callbacks::guest_response(ctx, ptr, len, h.clone())
      },
    ) {
      error!("Module did not import __guest_response");
      return Err("Module did not import __guest_response".into());
    }

    let h = host.clone();
    if let Err(_e) = module.link_closure(
      HOST_NAMESPACE,
      WapcFunctions::GUEST_ERROR_FN,
      move |ctx: &CallContext, (ptr, len): (i32, i32)| {
        callbacks::guest_error(ctx, ptr, len, h.clone())
      },
    ) {
      error!("Module did not import __guest_error");
      return Err("Module did not import __guest_error".into());
    }

    let h = host.clone();
    if let Err(_e) = module.link_closure(
      HOST_NAMESPACE,
      WapcFunctions::HOST_ERROR_FN,
      move |ctx: &CallContext, ptr: i32| callbacks::host_error(ctx, ptr, h.clone()),
    ) {
      warn!("Module did not import __host_error");
    }

    let h = host.clone();
    if let Err(_e) = module.link_closure(
      HOST_NAMESPACE,
      WapcFunctions::HOST_ERROR_LEN_FN,
      move |_ctx: &CallContext, ()| -> i32 { callbacks::host_error_length(h.clone()) },
    ) {
      warn!("Module did not import __host_error_len");
    }

    let _ = module.link_closure(
      WASI_UNSTABLE,
      "fd_write",
      move |_ctx: &CallContext, (_, _, _, _): (i32, i32, i32, i32)| -> i32 {
        warn!("Use of prohibited (WASI) fd_write function - suppressing output");
        0
      },
    ); // don't care if this function is missing

    // Fail the initialization if we can't find the guest call function
    if let Err(_e) = module.find_function::<(i32, i32), i32>(WapcFunctions::GUEST_CALL) {
      error!("Could not find __guest_call function in WebAssembly module");
      return Err(
        "Could not find __guest_call function in WebAssembly module"
          .to_string()
          .into(),
      );
    }

    // Invoke all the starters in order (if they exist)
    for starter in wapc::WapcFunctions::REQUIRED_STARTS.iter() {
      let func = module.find_function::<(), ()>(starter);
      if let Ok(func) = func {
        if let Err(e) = func.call() {
          error!(
            "Failed during invocation of starter function '{}': {}.",
            starter, e
          );
          return Err(format!("Failed during starter initialization '{}': {}", starter, e).into());
        }
      }
    }

    self.inner = Some(InnerProvider { rt, mod_name });

    Ok(())
  }

  fn call(&mut self, op_length: i32, msg_length: i32) -> Result<i32, Box<dyn Error>> {
    if let Some(ref i) = self.inner {
      let module = i.rt.find_module(&i.mod_name)?;
      let func = module.find_function::<(i32, i32), i32>(WapcFunctions::GUEST_CALL)?;
      let res = func.call(op_length, msg_length)?;
      Ok(res)
    } else {
      Err("Module call failure - no module was initialized".into())
    }
  }

  fn replace(&mut self, _bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    unimplemented!()
  }
}
