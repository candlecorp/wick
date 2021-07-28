use std::fs::read;
use wascc_codec::{deserialize, serialize};

use wapc::WapcHost;
#[test]
fn runs_wapc_guest() -> anyhow::Result<()> {
    let buf = read("tests/wasm/wapc_guest/test.wasm".to_string())?;

    let engine = wasmtime_provider::WasmtimeEngineProvider::new(&buf, None);
    let guest = WapcHost::new(Box::new(engine), move |_a, _b, _c, _d, _e| Ok(vec![]))?;

    let callresult = guest.call("echo", &serialize("hello world").unwrap())?;
    let result: String = deserialize(&callresult).unwrap();
    assert_eq!(result, "hello world");
    Ok(())
}
