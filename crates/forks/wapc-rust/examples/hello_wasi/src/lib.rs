// Copyright 2015-2019 Capital One Services, LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate wapc_guest as guest;

use guest::prelude::*;

wapc_handler!(handle_wapc);

pub fn handle_wapc(operation: &str, msg: &[u8]) -> CallResult {
    match operation {
        "wapc:sample!Hello" => hello_world(msg),
        _ => Err("bad dispatch".into()),
    }
}

// This function emits directly to the console, which requires a WASI host runtime
fn hello_world(msg: &[u8]) -> CallResult {
    let _res = host_call("myBinding", "wapc:sample", "Ping", msg)?;
    println!("Hello from inside the WASI module!");
    Ok(b"hello world!".to_vec())
}
