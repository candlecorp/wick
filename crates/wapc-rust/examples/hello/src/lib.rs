

extern crate wapc_guest as guest;

use guest::prelude::*;

wapc_handler!(handle_wapc);

pub fn handle_wapc(operation: &str, msg: &[u8]) -> CallResult {
    match operation {
        "wapc:sample!Hello" => hello_world(msg),
        _ => hello_world(b"Unknown operation")
    }
}

fn hello_world(msg: &[u8]) -> CallResult {
    guest::console_log(&format!(
        "Received message: {}",
        std::str::from_utf8(msg).unwrap()
    ));
    let _res = host_call("myBinding", "wapc:sample", "Ping", msg)?;
    Ok(b"hello world!".to_vec())
}
