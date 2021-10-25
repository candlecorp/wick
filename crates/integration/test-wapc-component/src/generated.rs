/**********************************************
***** This file is generated, do not edit *****
***********************************************/

use vino_provider::wasm::prelude::*;

type Result<T> = std::result::Result<T, WasmError>;

#[no_mangle]
pub(crate) extern "C" fn __guest_call(op_len: i32, req_len: i32) -> i32 {
  use std::slice;

  let buf: Vec<u8> = Vec::with_capacity(req_len as _);
  let req_ptr = buf.as_ptr();

  let opbuf: Vec<u8> = Vec::with_capacity(op_len as _);
  let op_ptr = opbuf.as_ptr();

  let (slice, op) = unsafe {
    wapc::__guest_request(op_ptr, req_ptr);
    (
      slice::from_raw_parts(req_ptr, req_len as _),
      slice::from_raw_parts(op_ptr, op_len as _),
    )
  };

  let op_str = ::std::str::from_utf8(op).unwrap();

  match Dispatcher::dispatch(op_str, slice) {
    Ok(response) => {
      unsafe { wapc::__guest_response(response.as_ptr(), response.len()) }
      1
    }
    Err(e) => {
      let errmsg = e.to_string();
      unsafe {
        wapc::__guest_error(errmsg.as_ptr(), errmsg.len() as _);
      }
      0
    }
  }
}

static ALL_COMPONENTS: &[&str] = &[
  "copy",
  "error",
  "reverse",
  "reverse-uppercase",
  "uppercase",
  "validate",
];

pub struct Dispatcher {}
impl Dispatch for Dispatcher {
  fn dispatch(op: &str, payload: &[u8]) -> CallResult {
    let payload = IncomingPayload::from_buffer(payload)?;
    let result = match op {
      "copy" => copy::Component::new().execute(&payload),
      "error" => error::Component::new().execute(&payload),
      "reverse" => reverse::Component::new().execute(&payload),
      "reverse-uppercase" => reverse_uppercase::Component::new().execute(&payload),
      "uppercase" => uppercase::Component::new().execute(&payload),
      "validate" => validate::Component::new().execute(&payload),
      _ => Err(WasmError::ComponentNotFound(
        op.to_owned(),
        ALL_COMPONENTS.join(", "),
      )),
    }?;
    Ok(serialize(&result)?)
  }
}

pub mod copy {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::wasm::prelude::*;

  use super::*;
  use crate::components::copy as implementation;

  pub struct Component {}

  impl Component {
    pub fn new() -> Self {
      Self {}
    }
  }
  impl WapcComponent for Component {
    fn execute(&self, payload: &IncomingPayload) -> JobResult {
      let outputs = get_outputs(payload.id());
      let inputs = populate_inputs(payload)?;
      implementation::job(inputs, outputs)
    }
  }

  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs> {
    Ok(Inputs {
      input: deserialize(payload.get("input")?)?,
      times: deserialize(payload.get("times")?)?,
    })
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("input".to_owned(), MessageTransport::success(&inputs.input));
      map.insert("times".to_owned(), MessageTransport::success(&inputs.times));
      map
    }
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
    #[serde(rename = "times")]
    pub times: i8,
  }

  #[derive(Debug)]
  pub struct OutputPorts {
    pub output: OutputSender,
  }

  #[derive(Debug, PartialEq, Clone)]
  pub struct OutputSender {
    id: u32,
  }

  impl PortSender for OutputSender {
    type PayloadType = String;
    fn get_name(&self) -> String {
      "output".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }

  fn get_outputs(id: u32) -> OutputPorts {
    OutputPorts {
      output: OutputSender { id },
    }
  }

  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| ComponentError::new("No packets for port 'output' found"))?;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub mod error {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::wasm::prelude::*;

  use super::*;
  use crate::components::error as implementation;

  pub struct Component {}

  impl Component {
    pub fn new() -> Self {
      Self {}
    }
  }
  impl WapcComponent for Component {
    fn execute(&self, payload: &IncomingPayload) -> JobResult {
      let outputs = get_outputs(payload.id());
      let inputs = populate_inputs(payload)?;
      implementation::job(inputs, outputs)
    }
  }

  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs> {
    Ok(Inputs {
      input: deserialize(payload.get("input")?)?,
    })
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("input".to_owned(), MessageTransport::success(&inputs.input));
      map
    }
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
  }

  #[derive(Debug)]
  pub struct OutputPorts {
    pub output: OutputSender,
  }

  #[derive(Debug, PartialEq, Clone)]
  pub struct OutputSender {
    id: u32,
  }

  impl PortSender for OutputSender {
    type PayloadType = String;
    fn get_name(&self) -> String {
      "output".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }

  fn get_outputs(id: u32) -> OutputPorts {
    OutputPorts {
      output: OutputSender { id },
    }
  }

  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| ComponentError::new("No packets for port 'output' found"))?;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub mod reverse {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::wasm::prelude::*;

  use super::*;
  use crate::components::reverse as implementation;

  pub struct Component {}

  impl Component {
    pub fn new() -> Self {
      Self {}
    }
  }
  impl WapcComponent for Component {
    fn execute(&self, payload: &IncomingPayload) -> JobResult {
      let outputs = get_outputs(payload.id());
      let inputs = populate_inputs(payload)?;
      implementation::job(inputs, outputs)
    }
  }

  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs> {
    Ok(Inputs {
      input: deserialize(payload.get("input")?)?,
    })
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("input".to_owned(), MessageTransport::success(&inputs.input));
      map
    }
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
  }

  #[derive(Debug)]
  pub struct OutputPorts {
    pub output: OutputSender,
  }

  #[derive(Debug, PartialEq, Clone)]
  pub struct OutputSender {
    id: u32,
  }

  impl PortSender for OutputSender {
    type PayloadType = String;
    fn get_name(&self) -> String {
      "output".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }

  fn get_outputs(id: u32) -> OutputPorts {
    OutputPorts {
      output: OutputSender { id },
    }
  }

  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| ComponentError::new("No packets for port 'output' found"))?;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub mod reverse_uppercase {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::wasm::prelude::*;

  use super::*;
  use crate::components::reverse_uppercase as implementation;

  pub struct Component {}

  impl Component {
    pub fn new() -> Self {
      Self {}
    }
  }
  impl WapcComponent for Component {
    fn execute(&self, payload: &IncomingPayload) -> JobResult {
      let outputs = get_outputs(payload.id());
      let inputs = populate_inputs(payload)?;
      implementation::job(inputs, outputs)
    }
  }

  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs> {
    Ok(Inputs {
      input: deserialize(payload.get("input")?)?,
      link: deserialize(payload.get("link")?)?,
    })
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("input".to_owned(), MessageTransport::success(&inputs.input));
      map.insert("link".to_owned(), MessageTransport::success(&inputs.link));
      map
    }
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
    #[serde(rename = "link")]
    pub link: ProviderLink,
  }

  #[derive(Debug)]
  pub struct OutputPorts {
    pub output: OutputSender,
  }

  #[derive(Debug, PartialEq, Clone)]
  pub struct OutputSender {
    id: u32,
  }

  impl PortSender for OutputSender {
    type PayloadType = String;
    fn get_name(&self) -> String {
      "output".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }

  fn get_outputs(id: u32) -> OutputPorts {
    OutputPorts {
      output: OutputSender { id },
    }
  }

  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| ComponentError::new("No packets for port 'output' found"))?;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub mod uppercase {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::wasm::prelude::*;

  use super::*;
  use crate::components::uppercase as implementation;

  pub struct Component {}

  impl Component {
    pub fn new() -> Self {
      Self {}
    }
  }
  impl WapcComponent for Component {
    fn execute(&self, payload: &IncomingPayload) -> JobResult {
      let outputs = get_outputs(payload.id());
      let inputs = populate_inputs(payload)?;
      implementation::job(inputs, outputs)
    }
  }

  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs> {
    Ok(Inputs {
      input: deserialize(payload.get("input")?)?,
    })
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("input".to_owned(), MessageTransport::success(&inputs.input));
      map
    }
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
  }

  #[derive(Debug)]
  pub struct OutputPorts {
    pub output: OutputSender,
  }

  #[derive(Debug, PartialEq, Clone)]
  pub struct OutputSender {
    id: u32,
  }

  impl PortSender for OutputSender {
    type PayloadType = String;
    fn get_name(&self) -> String {
      "output".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }

  fn get_outputs(id: u32) -> OutputPorts {
    OutputPorts {
      output: OutputSender { id },
    }
  }

  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| ComponentError::new("No packets for port 'output' found"))?;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub mod validate {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::wasm::prelude::*;

  use super::*;
  use crate::components::validate as implementation;

  pub struct Component {}

  impl Component {
    pub fn new() -> Self {
      Self {}
    }
  }
  impl WapcComponent for Component {
    fn execute(&self, payload: &IncomingPayload) -> JobResult {
      let outputs = get_outputs(payload.id());
      let inputs = populate_inputs(payload)?;
      implementation::job(inputs, outputs)
    }
  }

  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs> {
    Ok(Inputs {
      input: deserialize(payload.get("input")?)?,
    })
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("input".to_owned(), MessageTransport::success(&inputs.input));
      map
    }
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
  }

  #[derive(Debug)]
  pub struct OutputPorts {
    pub output: OutputSender,
  }

  #[derive(Debug, PartialEq, Clone)]
  pub struct OutputSender {
    id: u32,
  }

  impl PortSender for OutputSender {
    type PayloadType = String;
    fn get_name(&self) -> String {
      "output".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }

  fn get_outputs(id: u32) -> OutputPorts {
    OutputPorts {
      output: OutputSender { id },
    }
  }

  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| ComponentError::new("No packets for port 'output' found"))?;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
