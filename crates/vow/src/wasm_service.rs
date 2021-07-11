use std::collections::HashMap;
use std::convert::TryFrom;

use actix::prelude::*;
use vino_codec::messagepack::serialize;
use vino_component::Packet;
use vino_runtime::prelude::{
  MessageTransport,
  WapcModule,
};

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
#[rtype(result = "Result<HashMap<String, Packet>>")]
pub struct Call {
  pub component: String,
  pub message: MessageTransport,
}

impl Handler<Call> for WasmService {
  type Result = Result<HashMap<String, Packet>>;

  fn handle(&mut self, msg: Call, _ctx: &mut Self::Context) -> Self::Result {
    let message = msg.message.into_multibytes()?;
    let payload = serialize(("", message))?;
    self.host.call(&msg.component, &payload)
  }
}
