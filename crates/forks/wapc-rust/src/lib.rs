// !!START_LINTS
// Vino lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![deny(
    clippy::expect_used,
    clippy::explicit_deref_methods,
    clippy::option_if_let_else,
    clippy::await_holding_lock,
    clippy::cloned_instead_of_copied,
    clippy::explicit_into_iter_loop,
    clippy::flat_map_option,
    clippy::fn_params_excessive_bools,
    clippy::implicit_clone,
    clippy::inefficient_to_string,
    clippy::large_types_passed_by_value,
    clippy::manual_ok_or,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::must_use_candidate,
    clippy::needless_for_each,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::redundant_else,
    clippy::semicolon_if_nothing_returned,
    // clippy::too_many_lines,
    clippy::trivially_copy_pass_by_ref,
    clippy::unnested_or_patterns,
    clippy::future_not_send,
    clippy::useless_let_if_seq,
    clippy::str_to_string,
    clippy::inherent_to_string,
    clippy::let_and_return,
    clippy::string_to_string,
    clippy::try_err,
    clippy::if_then_some_else_none,
    bad_style,
    clashing_extern_declarations,
    const_err,
    // dead_code,
    deprecated,
    explicit_outlives_requirements,
    improper_ctypes,
    invalid_value,
    missing_copy_implementations,
    missing_debug_implementations,
    mutable_transmutes,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements ,
    patterns_in_fns_without_body,
    private_in_public,
    trivial_bounds,
    trivial_casts,
    trivial_numeric_casts,
    type_alias_bounds,
    unconditional_recursion,
    unreachable_pub,
    unsafe_code,
    unstable_features,
    // unused,
    unused_allocation,
    unused_comparisons,
    unused_import_braces,
    unused_parens,
    unused_qualifications,
    while_true,
    // missing_docs
)]
// !!END_LINTS
// Add exceptions here
#![allow(
  unused_qualifications,
  missing_copy_implementations,
  missing_debug_implementations
)]
#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/54989751?s=200&v=4")]

//! # wapc
//!
//! The `wapc` crate provides a high-level WebAssembly host runtime that conforms to an RPC mechanism
//! called **waPC**. waPC is designed to be a fixed, lightweight standard allowing both sides of the
//! guest/host boundary to make method calls containing arbitrary binary payloads. Neither side
//! of the contract is ever required to perform explicit allocation, ensuring maximum portability
//! for wasm targets that might behave differently in the presence of garbage collectors and memory
//! relocation, compaction, etc.
//!
//! The interface may at first appear more "chatty" than other protocols, but the cleanliness, ease of use,
//! simplified developer experience, and purpose-fit aim toward stateless WebAssembly modules
//! is worth the few extra nanoseconds of latency.
//!
//! To use `wapc`, first you'll need a waPC-compliant WebAssembly module (referred to as the _guest_) to load
//! and execute. You can find a number of these samples available in the GitHub repository,
//! and anything compiled with the [wascc](https://github.com/wascc) actor SDK can also be invoked
//! via waPC as it is 100% waPC compliant.
//!
//! Next, you will need to chose a _runtime engine_. waPC describes the function call flow required
//! for wasm-RPC, but it does not dictate how the low-level WebAssembly function calls are made. This
//! allows you to select whatever engine best suits your needs, whether it's a JIT-based engine or an
//! interpreter-based one. Simply instantiate anything that implements the
//! [WebAssemblyEngineProvider](trait.WebAssemblyEngineProvider.html) trait and pass it to the WapcHost
//! constructor and the [WapcHost](struct.WapcHost.html) will facilitate all RPCs.
//!
//! To make function calls, ensure that you provided a suitable host callback function (or closure)
//! when you created your WapcHost. Then invoke the `call` function to initiate the RPC flow.
//!
//! # Example
//!
//! ```ignore
//! extern crate wapc;
//! use wapc::prelude::*;
//! use wasmtime_provider::WasmtimeEnginerProvider; // Choose your own engine provider
//!
//! # fn load_file() -> Vec<u8> {
//! #    include_bytes!("../.assets/hello.wasm").to_vec()
//! # }
//! # fn load_wasi_file() -> Vec<u8> {
//! #    include_bytes!("../.assets/hello_wasi.wasm").to_vec()
//! # }
//! pub fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//!     let module_bytes = load_file();
//!     let engine = WasmtimeEngineProvider::new(&bytes, None);
//!     let host = WapcHost::new(
//!                 Box::new(engine),
//!                 |id: u64, bd: &str, ns: &str, op: &str, payload: &[u8]| {
//!                   println!("Guest {} invoked '{}->{}:{}' with payload of {} bytes",
//!                             id, bd, ns, op, payload.len());
//!                   Ok(vec![])
//!                 })?;
//!
//!     let res = host.call("wapc:sample!Hello", b"this is a test")?;
//!     assert_eq!(res, b"hello world!");
//!
//!     Ok(())
//! }
//! ```
//!
//! # Notes
//!
//! waPC is _reactive_. Guest modules cannot initiate host calls without first handling a call
//! initiated by the host. It is up to the runtime engine provider (e.g. `wasmtime` or `wasm3`)
//! to invoke the required start functions (if present) during initialization. Guest modules can
//! synchronously make as many host calls as they like, but keep in mind that if a host call takes too long or fails, it'll cause the initiating
//! guest call to also fail.
//!
//! In summary, keep host callbacks fast and and free of panic-friendly `unwrap()`s, and do not spawn new threads
//! within a host callback unless you must (and can synchronously return a value) because waPC
//! assumes a single-threaded execution environment. Also note that for safety the host callback function
//! intentionally has no references to the WebAssembly module bytes or the running instance. If you need
//! an external reference in the callback, you can capture it in a closure.
//!
//! ## RPC Exchange Flow
//!
//! The following is a detailed outline of which functions are invoked and in which order to support
//! a waPC exchange flow, which is always triggered by a consumer invoking the `call` function. Invoking
//! and handling these low-level functions is the responsibility of the _engine provider_, while
//! orchestrating the high-level control flow is the job of the `WapcHost`.
//!
//! 1. Host invokes `__guest_call` on the WebAssembly module (via the engine provider)
//! 1. Guest calls the `__guest_request` function to instruct the host to write the request parameters to linear memory
//! 1. Guest uses the `op_len` and `msg_len` parameters long with the pointer values it generated in step 2 to retrieve the operation (UTF-8 string) and payload (opaque byte array)
//! 1. Guest performs work
//! 1. _(Optional)_ Guest invokes `__host_call` on host with pointers and lengths indicating the `binding`, `namespace`, `operation`, and payload.
//! 1. _(Optional)_ Guest can use `__host_response` and `host_response_len` functions to obtain and interpret results
//! 1. _(Optional)_ Guest can use `__host_error_len` and `__host_error` to obtain the host error if indicated (`__host_call` returns 0)
//!     1. Steps 5-7 can repeat with as many different host calls as the guest needs
//! 1. Guest will call `guest_error` to indicate if an error occurred during processing
//! 1. Guest will call `guest_response` to store the opaque response payload
//! 1. Guest will return 0 (error) or 1 (success) at the end of `__guest_call`
//!
//! ## Required Host Exports
//! List of functions that must be exported by the host (imported by the guest)
//!
//! | Module         | Function       | Parameters      | Description                             |
//! |----------------|----------------|-----------------|-----------------------------------------|
//! | wapc           | __host_call    | br_ptr: i32<br/>bd_len: i32<br/>ns_ptr: i32<br/>ns_len: i32<br/>op_ptr: i32<br/>op_len: i32<br/>ptr: i32<br/>len: i32<br/>-> i32     | Invoked to initiate a host call         |
//! | wapc           | __console_log  | ptr: i32, len: i32 | Allows guest to log to `stdout` |
//! | wapc           | __guest_request | op_ptr: i32<br/>ptr: i32 | Writes the guest request payload and operation name to linear memory at the designated locations |
//! | wapc           | __host_response | ptr: i32 | Instructs host to write the host response payload to the given location in linear memory |
//! | wapc           | __host_response_len | -> i32 | Obtains the length of the current host response |
//! | wapc | __guest_response | ptr: i32<br/>len: i32 | Tells the host the size and location of the current guest response payload |
//! | wapc | __guest_error | ptr: i32<br/>len: i32 | Tells the host the size and location of the current guest error payload |
//! | wapc | __host_error | ptr: i32 | Instructs the host to write the host error payload to the given location |
//! | wapc | __host_error_len | -> i32 | Queries the host for the length of the current host error (0 if none) |
//!
//!
//! ## Required Guest Exports
//! List of functions that must be exported by the guest (invoked by the host)
//!
//! | Function | Parameters | Description |
//! |----------|------------|-------------|
//! | __guest_call | op_len: i32<br/>msg_len: i32 | Invoked by the host to start an RPC exchange with the guest module |

pub mod errors;

/// A result type for errors that occur within the wapc library
pub type Result<T> = std::result::Result<T, errors::Error>;

type HostType = Arc<Mutex<ModuleState>>;

use std::cell::RefCell;
use std::error::Error;
use std::sync::atomic::{
  AtomicU64,
  Ordering,
};
use std::sync::{
  Arc,
  RwLock,
};

use parking_lot::Mutex;

static GLOBAL_MODULE_COUNT: AtomicU64 = AtomicU64::new(1);

/// The host module name / namespace that guest modules must use for imports
pub const HOST_NAMESPACE: &str = "wapc";

/// A list of the function names that are part of each waPC conversation
pub struct WapcFunctions;

impl WapcFunctions {
  // -- Functions called by guest, exported by host
  pub const HOST_CONSOLE_LOG: &'static str = "__console_log";
  pub const HOST_CALL: &'static str = "__host_call";
  pub const GUEST_REQUEST_FN: &'static str = "__guest_request";
  pub const HOST_RESPONSE_FN: &'static str = "__host_response";
  pub const HOST_RESPONSE_LEN_FN: &'static str = "__host_response_len";
  pub const GUEST_RESPONSE_FN: &'static str = "__guest_response";
  pub const GUEST_ERROR_FN: &'static str = "__guest_error";
  pub const HOST_ERROR_FN: &'static str = "__host_error";
  pub const HOST_ERROR_LEN_FN: &'static str = "__host_error_len";

  // -- Functions called by host, exported by guest
  pub const GUEST_CALL: &'static str = "__guest_call";
  pub const WAPC_INIT: &'static str = "wapc_init";
  pub const TINYGO_START: &'static str = "_start";

  /// Start functions to attempt to call - order is important
  pub const REQUIRED_STARTS: [&'static str; 2] = [Self::TINYGO_START, Self::WAPC_INIT];
}

/// Parameters defining the options for enabling WASI on a module (if applicable)
#[derive(Debug, Default)]
#[must_use]
pub struct WasiParams {
  pub argv: Vec<String>,
  pub map_dirs: Vec<(String, String)>,
  pub env_vars: Vec<(String, String)>,
  pub preopened_dirs: Vec<String>,
}

impl WasiParams {
  pub fn new(
    argv: Vec<String>,
    map_dirs: Vec<(String, String)>,
    env_vars: Vec<(String, String)>,
    preopened_dirs: Vec<String>,
  ) -> Self {
    WasiParams {
      argv,
      map_dirs,
      preopened_dirs,
      env_vars,
    }
  }
}

#[derive(Default)]
/// Module state is essentially a 'handle' that is passed to a runtime engine to allow it
/// to read and write relevant data as different low-level functions are executed during
/// a waPC conversation
pub struct ModuleState {
  guest_request: RwLock<Option<Invocation>>,
  guest_response: RwLock<Option<Vec<u8>>>,
  host_response: RwLock<Option<Vec<u8>>>,
  guest_error: RwLock<Option<String>>,
  host_error: RwLock<Option<String>>,
  host_callback: Option<Box<HostCallback>>,
  id: u64,
}

impl ModuleState {
  pub(crate) fn new(host_callback: Box<HostCallback>, id: u64) -> ModuleState {
    ModuleState {
      host_callback: Some(Box::new(host_callback)),
      id,
      guest_request: RwLock::new(None),
      guest_response: RwLock::new(None),
      host_response: RwLock::new(None),
      guest_error: RwLock::new(None),
      host_error: RwLock::new(None),
    }
  }
}

impl ModuleState {
  /// Retrieves the value, if any, of the current guest request
  pub fn get_guest_request(&self) -> Option<Invocation> {
    self.guest_request.read().unwrap().clone()
  }

  /// Retrieves the value of the current host response
  pub fn get_host_response(&self) -> Option<Vec<u8>> {
    self.host_response.read().unwrap().clone()
  }

  /// Sets a value indicating that an error occurred inside the execution of a guest call
  pub fn set_guest_error(&self, error: String) {
    *self.guest_error.write().unwrap() = Some(error);
  }

  /// Sets the value indicating the response data from a guest call
  pub fn set_guest_response(&self, response: Vec<u8>) {
    *self.guest_response.write().unwrap() = Some(response);
  }

  /// Queries the value of the current guest response
  pub fn get_guest_response(&self) -> Option<Vec<u8>> {
    self.guest_response.read().unwrap().clone()
  }

  /// Queries the value of the current host error
  pub fn get_host_error(&self) -> Option<String> {
    self.host_error.read().unwrap().clone()
  }

  /// Invoked when the guest module wishes to make a call on the host
  pub fn do_host_call(
    &self,
    binding: &str,
    namespace: &str,
    operation: &str,
    payload: &[u8],
  ) -> std::result::Result<i32, Box<dyn Error>> {
    let id = {
      *self.host_response.write().unwrap() = None;
      *self.host_error.write().unwrap() = None;
      self.id
    };
    let result = {
      match self.host_callback {
        Some(ref f) => f(id, binding, namespace, operation, payload),
        None => Err("Missing host callback function!".into()),
      }
    };
    Ok(match result {
      Ok(v) => {
        *self.host_response.write().unwrap() = Some(v);
        1
      }
      Err(e) => {
        *self.host_error.write().unwrap() = Some(format!("{}", e));
        0
      }
    })
  }

  /// Invoked when the guest module wants to write a message to the host's `stdout`
  pub fn do_console_log(&self, msg: &str) {
    println!("{}", msg);
  }

  pub fn set_host_callback(&mut self, callback: Box<HostCallback>) {
    self.host_callback = Some(callback);
  }
}

/// An engine provider is any code that encapsulates low-level WebAssembly interactions such
/// as reading from and writing to linear memory, executing functions, and mapping imports
/// in a way that conforms to the waPC conversation protocol.
pub trait WebAssemblyEngineProvider {
  /// Tell the engine provider that it can do whatever processing it needs to do for
  /// initialization and give it access to the module state
  fn init(&mut self, host: HostType) -> std::result::Result<(), Box<dyn std::error::Error>>;
  /// Trigger the waPC function call. Engine provider is responsible for execution and using the appropriate methods
  /// on the module host. When this function is complete, the guest response and optionally the guest
  /// error must be set to represent the high-level call result
  fn call(
    &mut self,
    op_length: i32,
    msg_length: i32,
  ) -> std::result::Result<i32, Box<dyn std::error::Error>>;
  /// Called by the host to replace the WebAssembly module bytes of the previously initialized module. Engine must return an
  /// error if it does not support bytes replacement.
  fn replace(&mut self, bytes: &[u8]) -> std::result::Result<(), Box<dyn std::error::Error>>;
}

/// The module host (waPC) must provide an implementation of this trait to the engine provider
/// to enable waPC function calls.
pub trait ModuleHost {
  /// Called by the engine provider to obtain the Invocation bound for the guest module
  fn get_guest_request(&self) -> Option<Invocation>;
  /// Called by the engine provider to query the results of a host function call
  fn get_host_response(&self) -> Option<Vec<u8>>;
  /// Called by the engine provider to set the error message indicating a failure that occurred inside the guest module execution
  fn set_guest_error(&self, error: String);
  /// Called by the engine provider to set the response data for a guest call
  fn set_guest_response(&self, response: Vec<u8>);
  /// Called by the engine provider to query the host error if one is indicated by the return code for a host call
  fn get_host_error(&self) -> Option<String>;
  /// Called by the engine provider to allow a guest module to perform a host call. The numeric return value
  /// will be > 0 for success (engine must obtain the host response) or 0 for error (engine must obtain the error)
  fn do_host_call(
    &self,
    binding: &str,
    namespace: &str,
    operation: &str,
    payload: &[u8],
  ) -> std::result::Result<i32, Box<dyn std::error::Error>>;
  /// Attempts to perform a console log. There are no guarantees this will happen, and no error will be returned
  /// to the guest module if the host rejects the attempt
  fn do_console_log(&self, msg: &str);
}

type HostCallback = dyn Fn(
    u64,
    &str,
    &str,
    &str,
    &[u8],
  ) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>
  + Sync
  + Send
  + 'static;

#[derive(Debug, Clone)]
/// Represents a waPC invocation, which is a combination of an operation string and the
/// corresponding binary payload
pub struct Invocation {
  pub operation: String,
  pub msg: Vec<u8>,
}

impl Invocation {
  /// Creates a new invocation
  fn new(op: &str, msg: Vec<u8>) -> Invocation {
    Invocation {
      operation: op.to_owned(),
      msg,
    }
  }
}

/// A WebAssembly host runtime for waPC-compliant modules
///
/// Use an instance of this struct to provide a means of invoking procedure calls by
/// specifying an operation name and a set of bytes representing the opaque operation payload.
/// `WapcHost` makes no assumptions about the contents or format of either the payload or the
/// operation name, other than that the operation name is a UTF-8 encoded string.
pub struct WapcHost {
  engine: RefCell<Box<dyn WebAssemblyEngineProvider>>,
  state: HostType,
}

impl std::fmt::Debug for WapcHost {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("WapcHost <ignored>")
  }
}

impl WapcHost {
  /// Creates a new instance of a waPC-compliant host runtime paired with a given
  /// low-level engine provider
  pub fn new(
    engine: Box<dyn WebAssemblyEngineProvider>,
    host_callback: impl Fn(
        u64,
        &str,
        &str,
        &str,
        &[u8],
      ) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>
      + 'static
      + Sync
      + Send,
  ) -> Result<Self> {
    let id = GLOBAL_MODULE_COUNT.fetch_add(1, Ordering::SeqCst);
    // let state = Rc::new(RefCell::new(ModuleState::new(id, Box::new(host_callback))));
    let state = Arc::new(Mutex::new(ModuleState::new(Box::new(host_callback), id)));

    let mh = WapcHost {
      engine: RefCell::new(engine),
      state: state.clone(),
    };

    mh.initialize(state)?;

    Ok(mh)
  }

  pub fn set_callback(
    &mut self,
    host_callback: impl Fn(
        u64,
        &str,
        &str,
        &str,
        &[u8],
      ) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>
      + 'static
      + Sync
      + Send,
  ) {
    self.state.lock().set_host_callback(Box::new(host_callback));
  }

  fn initialize(&self, state: HostType) -> Result<()> {
    match self.engine.borrow_mut().init(state) {
      Ok(_) => Ok(()),
      Err(e) => Err(crate::errors::new(
        crate::errors::ErrorKind::GuestCallFailure(format!(
          "Failed to initialize guest module: {}",
          e
        )),
      )),
    }
  }

  /// Returns a reference to the unique identifier of this module. If a parent process
  /// has instantiated multiple `WapcHost`s, then the single static host callback function
  /// will contain this value to allow disambiguation of modules
  pub fn id(&self) -> u64 {
    self.state.lock().id
  }

  /// Invokes the `__guest_call` function within the guest module as per the waPC specification.
  /// Provide an operation name and an opaque payload of bytes and the function returns a `Result`
  /// containing either an error or an opaque reply of bytes.
  ///
  /// It is worth noting that the _first_ time `call` is invoked, the WebAssembly module
  /// might incur a "cold start" penalty, depending on which underlying engine you're using. This
  /// might be due to lazy initialization or JIT-compilation.
  pub fn call(&self, op: &str, payload: &[u8]) -> Result<Vec<u8>> {
    let inv = Invocation::new(op, payload.to_vec());

    {
      *self.state.lock().guest_response.write().unwrap() = None;
      *self.state.lock().guest_request.write().unwrap() = Some((inv).clone());
      *self.state.lock().guest_error.write().unwrap() = None;
      *self.state.lock().host_response.write().unwrap() = None;
      *self.state.lock().host_error.write().unwrap() = None;
    }

    let callresult = match self
      .engine
      .borrow_mut()
      .call(inv.operation.len() as i32, inv.msg.len() as i32)
    {
      Ok(c) => c,
      Err(e) => {
        return Err(errors::new(errors::ErrorKind::GuestCallFailure(format!(
          "{}",
          e
        ))));
      }
    };

    let state = self.state.lock();

    if callresult == 0 {
      // invocation failed
      let lock = state.guest_error.read().unwrap();
      match *lock {
        Some(ref s) => Err(errors::new(errors::ErrorKind::GuestCallFailure(s.clone()))),
        None => Err(errors::new(errors::ErrorKind::GuestCallFailure(
          "No error message set for call failure".to_owned(),
        ))),
      }
    } else {
      // invocation succeeded
      match *state.guest_response.read().unwrap() {
        Some(ref e) => Ok(e.clone()),
        None => {
          let lock = state.guest_error.read().unwrap();
          match *lock {
            Some(ref s) => Err(errors::new(errors::ErrorKind::GuestCallFailure(s.clone()))),
            None => Err(errors::new(errors::ErrorKind::GuestCallFailure(
              "No error message OR response set for call success".to_owned(),
            ))),
          }
        }
      }
    }
  }

  /// Performs a live "hot swap" of the WebAssembly module. Since all internal waPC execution is assumed to be
  /// single-threaded and non-reentrant, this call is synchronous and so
  /// you should never attempt to invoke `call` from another thread while performing this hot swap.
  ///
  /// **Note**: if the underlying engine you've chosen is a JITting engine, then performing a swap
  /// will re-introduce a "cold start" delay upon the next function call.
  ///
  /// If you perform a hot swap of a WASI module, you cannot alter the parameters used to create the WASI module
  /// like the environment variables, mapped directories, pre-opened files, etc. Not abiding by this could lead
  /// to privilege escalation attacks or non-deterministic behavior after the swap.
  pub fn replace_module(&self, module: &[u8]) -> Result<()> {
    match self.engine.borrow_mut().replace(module) {
      Ok(_) => Ok(()),
      Err(e) => Err(errors::new(errors::ErrorKind::GuestCallFailure(format!(
        "Failed to swap module bytes: {}",
        e
      )))),
    }
  }
}
