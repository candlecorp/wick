![Rust](https://github.com/wapc/wapc-rust/workflows/Rust/badge.svg)
![crates.io](https://img.shields.io/crates/v/wapc.svg)
![license](https://img.shields.io/crates/l/wapc.svg)

# waPC

This is the Rust implementation of the **waPC** standard for WebAssembly host runtimes. It allows any WebAssembly module to be loaded as a guest and receive requests for invocation as well as to make its own function requests of the host. This library allows for both "pure" (completely isolated) wasm modules as well as WASI modules

This crate defines the protocol for RPC exchange between guest (WebAssembly) modules and the host runtime. That protocol
can be satisfied by any engine that implements the right trait. This allows you to choose the WebAssembly
low-level "driver" that best suits your needs, whether it be JITted or interpreted or bespoke.

## Example

The following is a simple example of synchronous, bi-directional procedure calls between a WebAssembly host runtime and the guest module.

```rust
extern crate wapc;
use wasmtime_provider::WasmtimeEngineProvider;
use wapc::prelude::*;

pub fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let module = load_file();
    let engine = WasmtimeEngineProvider::new(&module, None);
    let mut host = WapcHost::new(
       engine,
       |id: u64, bd: &str, ns: &str, op: &str, payload: &str|{
        println!("Guest {} invoked '{}->{}:{}' with payload of {} bytes", id, bd, ns, op, payload.len());
        Ok(vec![])
    }, &module, None)?;

    let res = host.call("wapc:sample!Hello", b"this is a test")?;
    assert_eq!(res, b"hello world!");
    Ok(())
}
```

For running examples, take a look at the examples available in the individual engine provider
repositories:

* [wasmtime-provider](https://github.com/wapc/wasmtime-provider) - Utilizes the [Bytecode Alliance](https://bytecodealliance.org/) runtime [wasmtime](https://github.com/bytecodealliance/wasmtime) for WebAssembly JIT compilation and execution.
* [wasm3-provider](https://github.com/wapc/wasm3-provider) - Uses the [wasm3](https://github.com/wasm3) C interpreter runtime (with a [Rust wrapper](https://github.com/Veykril/wasm3-rs))
