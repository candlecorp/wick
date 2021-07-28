use std::fs::read;

use wapc::WapcHost;

fn create_guest(path: String) -> Result<WapcHost, wapc::errors::Error> {
    let buf = read(path)?;

    let engine = wasmtime_provider::WasmtimeEngineProvider::new(&buf, None);
    WapcHost::new(Box::new(engine), move |_a, _b, _c, _d, _e| Ok(vec![]))
}

#[test]
fn runs_hello() -> anyhow::Result<()> {
    let guest = create_guest("./.assets/hello.wasm".to_string())?;

    let callresult = guest.call("wapc:sample!Hello", b"this is a test")?;
    let result = String::from_utf8_lossy(&callresult);
    assert_eq!(result, "hello world!");
    Ok(())
}

#[test]
fn runs_hello_as() -> anyhow::Result<()> {
    let guest = create_guest("./.assets/hello_as.wasm".to_string())?;

    let callresult = guest.call("hello", b"this is a test")?;
    let result = String::from_utf8_lossy(&callresult);
    assert_eq!(result, "Hello");
    Ok(())
}

#[test]
fn runs_hello_tinygo() -> anyhow::Result<()> {
    let guest = create_guest("./.assets/hello_tinygo.wasm".to_string())?;

    let callresult = guest.call("hello", b"this is a test")?;
    let result = String::from_utf8_lossy(&callresult);
    assert_eq!(result, "Hello");
    Ok(())
}

#[test]
fn runs_hello_wasi() -> anyhow::Result<()> {
    let guest = create_guest("./.assets/hello_wasi.wasm".to_string())?;

    let callresult = guest.call("wapc:sample!Hello", b"this is a test")?;
    println!("{:?}", callresult);
    let result = String::from_utf8_lossy(&callresult);
    assert_eq!(result, "hello world!");
    Ok(())
}

#[test]
fn runs_hello_zig() -> anyhow::Result<()> {
    let guest = create_guest("./.assets/hello_zig.wasm".to_string())?;

    let callresult = guest.call("hello", b"this is a test")?;
    let result = String::from_utf8_lossy(&callresult);
    assert_eq!(result, "Hello, this is a test!");
    Ok(())
}
