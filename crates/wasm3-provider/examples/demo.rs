extern crate wapc;

use std::fs::File;
use std::io::prelude::*;

use std::time::Instant;
use wapc::WapcHost;
use wasm3_provider::Wasm3EngineProvider;

fn load_file(path: &str) -> Vec<u8> {
    println!("{}", path);
    let mut f = File::open(path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    buf
}

pub fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let n = Instant::now();
    let module_bytes = load_file(&std::env::args().skip(1).next().unwrap());
    let engine = Wasm3EngineProvider::new(&module_bytes);
    let host = WapcHost::new(Box::new(engine), host_callback)?;

    let func = std::env::args().skip(2).next().unwrap();

    // hello.wasm - operation is wapc:sample!Hello (use ' quotes for linux CLI)
    // hello_as.wasm - operation is hello
    // hello_tinygo.wasm - operation is hello
    // hello_zig.wasm - operation is hello
    println!("Calling guest (wasm) function");
    let res = host.call(&func, b"this is a test")?;
    println!("Result - {}", ::std::str::from_utf8(&res).unwrap());

    println!("Elapsed - {}ms", n.elapsed().as_millis());
    Ok(())
}

fn host_callback(
    id: u64,
    bd: &str,
    ns: &str,
    op: &str,
    payload: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    println!(
        "Guest {} invoked '{}->{}:{}' with payload of {}",
        id,
        bd,
        ns,
        op,
        ::std::str::from_utf8(payload).unwrap()
    );
    Ok(vec![])
}
