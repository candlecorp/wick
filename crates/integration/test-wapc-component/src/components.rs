/**********************************************
***** This file is generated, do not edit *****
***********************************************/

pub use vino_provider::prelude::*;

pub mod copy; // copy
pub mod error; // error
pub mod reverse; // reverse
pub mod reverse_uppercase; // reverse-uppercase
pub mod uppercase; // uppercase
pub mod validate; // validate

pub mod __multi__;

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

static ALL_COMPONENTS: &[&str] = &["copy", "error", "reverse", "reverse-uppercase", "uppercase", "validate"];

pub struct Dispatcher {}
impl Dispatch for Dispatcher {
  fn dispatch(op: &str, payload: &[u8]) -> CallResult {
    let payload = IncomingPayload::from_buffer(payload)?;
    let result = match op {
      "copy" => crate::components::generated::copy::Component::default().execute(&payload),
      "error" => crate::components::generated::error::Component::default().execute(&payload),
      "reverse" => crate::components::generated::reverse::Component::default().execute(&payload),
      "reverse-uppercase" => crate::components::generated::reverse_uppercase::Component::default().execute(&payload),
      "uppercase" => crate::components::generated::uppercase::Component::default().execute(&payload),
      "validate" => crate::components::generated::validate::Component::default().execute(&payload),
      _ => Err(WasmError::ComponentNotFound(op.to_owned(), ALL_COMPONENTS.join(", "))),
    }?;
    Ok(serialize(&result)?)
  }
}

pub mod types {
  // no additional types
}

pub mod generated {
  use super::*;

  // start namespace
  // Leaf namespace

  // Sub-components

  pub mod copy {
    use crate::components::copy as implementation;

    pub use vino_provider::prelude::*;

    use super::*;

    #[derive(Default)]
    pub struct Component {}

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

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
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
    use crate::components::error as implementation;

    pub use vino_provider::prelude::*;

    use super::*;

    #[derive(Default)]
    pub struct Component {}

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

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
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
    use crate::components::reverse as implementation;

    pub use vino_provider::prelude::*;

    use super::*;

    #[derive(Default)]
    pub struct Component {}

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

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
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
    use crate::components::reverse_uppercase as implementation;

    pub use vino_provider::prelude::*;

    use super::*;

    #[derive(Default)]
    pub struct Component {}

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

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
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
    use crate::components::uppercase as implementation;

    pub use vino_provider::prelude::*;

    use super::*;

    #[derive(Default)]
    pub struct Component {}

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

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
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
    use crate::components::validate as implementation;

    pub use vino_provider::prelude::*;

    use super::*;

    #[derive(Default)]
    pub struct Component {}

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

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
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

  pub mod __multi__ {
    use super::Result;
    use crate::components::__multi__ as implementation;

    #[cfg(any(feature = "native"))]
    pub use vino_provider::native::prelude::*;
    #[cfg(any(feature = "wasm"))]
    pub use vino_provider::wasm::prelude::*;

    pub use vino_provider::prelude::*;
    #[derive(Default)]
    pub struct Component {}

    impl WapcComponent for Component {
      fn execute(&self, payload: &IncomingPayload) -> JobResult {
        let outputs = get_outputs(payload.id());
        let inputs = populate_inputs(payload)?;
        implementation::job(inputs, outputs)
      }
    }

    fn populate_inputs(payload: &IncomingPayload) -> Result<Vec<ComponentInputs>> {
      Ok(deserialize(payload.get("inputs")?)?)
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub enum ComponentInputs {
      Copy(super::copy::Inputs),
      Error(super::error::Inputs),
      Reverse(super::reverse::Inputs),
      ReverseUppercase(super::reverse_uppercase::Inputs),
      Uppercase(super::uppercase::Inputs),
      Validate(super::validate::Inputs),
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub enum ComponentOutputs {
      Copy(super::copy::Outputs),
      Error(super::error::Outputs),
      Reverse(super::reverse::Outputs),
      ReverseUppercase(super::reverse_uppercase::Outputs),
      Uppercase(super::uppercase::Outputs),
      Validate(super::validate::Outputs),
    }

    #[derive(Debug)]
    pub struct OutputPorts {
      pub result: ResultSender,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct ResultSender {
      id: u32,
    }

    impl PortSender for ResultSender {
      type PayloadType = bool;
      fn get_name(&self) -> String {
        "result".to_string()
      }
      fn get_id(&self) -> u32 {
        self.id
      }
    }

    fn get_outputs(id: u32) -> OutputPorts {
      OutputPorts {
        result: ResultSender { id },
      }
    }

    #[derive(Debug)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    impl Outputs {
      pub fn result(&mut self) -> Result<PortOutput> {
        let packets = self
          .packets
          .take("result")
          .ok_or_else(|| ComponentError::new("No packets for port 'result' found"))?;
        Ok(PortOutput::new("result".to_owned(), packets))
      }
    }

    impl From<ProviderOutput> for Outputs {
      fn from(packets: ProviderOutput) -> Self {
        Self { packets }
      }
    }
  }
}
