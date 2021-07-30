use std::collections::HashMap;
use std::convert::TryFrom;

use actix::prelude::*;
use vino_codec::messagepack::serialize;
use vino_transport::MessageTransportStream;
use vino_types::signatures::ComponentSignature;

use crate::wapc_module::WapcModule;
use crate::wasm_host::WasmHost;
use crate::Result;

impl Actor for WasmService {
  type Context = SyncContext<Self>;
}

#[derive(Debug)]
pub struct WasmService {
  host: WasmHost,
}

impl WasmService {
  #[must_use]
  pub fn new(module: &WapcModule) -> Self {
    Self {
      host: WasmHost::try_from(module).unwrap(),
    }
  }
}

#[derive(Message)]
#[rtype(result = "()")]
struct Initialize {
  host: WasmHost,
}

impl Handler<Initialize> for WasmService {
  type Result = ();

  fn handle(&mut self, msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {
    self.host = msg.host;
  }
}

#[derive(Message, Debug)]
#[rtype(result = "Result<MessageTransportStream>")]
pub struct Call {
  pub component: String,
  pub payload: HashMap<String, Vec<u8>>,
}

impl Handler<Call> for WasmService {
  type Result = Result<MessageTransportStream>;

  fn handle(&mut self, msg: Call, _ctx: &mut Self::Context) -> Self::Result {
    let payload = ("", msg.payload);
    let payload = serialize(&payload)?;
    self.host.call(&msg.component, &payload)
  }
}

#[derive(Message, Copy, Clone, Debug)]
#[rtype(result = "Result<Vec<ComponentSignature>>")]
pub struct GetComponents {}

impl Handler<GetComponents> for WasmService {
  type Result = Result<Vec<ComponentSignature>>;

  fn handle(&mut self, _msg: GetComponents, _ctx: &mut Self::Context) -> Self::Result {
    Ok(self.host.get_components().clone())
  }
}
