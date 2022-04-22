/**********************************************
***** This file is generated, do not edit *****
***********************************************/

#[cfg(all(feature = "native", not(feature = "wasm")))]
pub use vino_provider::native::prelude::*;
#[cfg(all(feature = "wasm", not(feature = "native")))]
pub use vino_provider::wasm::prelude::*;

pub mod __multi__;
pub mod main; // main

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

static ALL_COMPONENTS: &[&str] = &["main"];

pub struct Dispatcher {}
impl Dispatch for Dispatcher {
  fn dispatch(op: &str, payload: &[u8]) -> CallResult {
    let payload = IncomingPayload::from_buffer(payload)?;
    let result = match op {
      "main" => crate::components::generated::main::Component::default().execute(&payload),
      _ => Err(WasmError::ComponentNotFound(op.to_owned(), ALL_COMPONENTS.join(", "))),
    }?;
    Ok(serialize(&result)?)
  }
}

pub mod types {
  // no additional types
}

pub mod generated {

  // start namespace
  pub mod main {
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    use crate::components::main as implementation;

    #[derive(Default)]
    pub struct Component {}

    impl WapcComponent for Component {
      fn execute(&self, payload: &IncomingPayload) -> JobResult {
        let outputs = get_outputs(payload.id());
        let inputs = populate_inputs(payload)?;
        implementation::job(inputs, outputs)
      }
    }

    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
      Ok(Inputs {
        argv: payload.consume("argv")?,
      })
    }

    #[cfg(all(feature = "wasm", not(feature = "native")))]
    fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
      Ok(Inputs {
        argv: deserialize(payload.get("argv")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "argv")]
      pub argv: Vec<String>,
    }

    #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
    impl From<Inputs> for TransportMap {
      fn from(inputs: Inputs) -> TransportMap {
        let mut map = TransportMap::new();
        map.insert("argv", MessageTransport::success(&inputs.argv));
        map
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", feature = "native"))]
    pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert(
        "argv".to_owned(),
        TypeSignature::List {
          element: Box::new(TypeSignature::String),
        },
      );
      map
    }
    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(all(feature = "provider", any(feature = "native", feature = "wasm")))]
    pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("code".to_owned(), TypeSignature::U32);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
    #[cfg(feature = "provider")]
    pub struct OutputPorts {
      pub code: CodePortSender,
    }

    // Definition and implementation of each port's sender.
    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct CodePortSender {
      #[cfg(feature = "native")]
      port: PortChannel,
      #[cfg(feature = "wasm")]
      id: u32,
    }

    #[cfg(all(feature = "provider", feature = "native"))]
    impl Default for CodePortSender {
      fn default() -> Self {
        Self {
          port: PortChannel::new("code"),
        }
      }
    }

    // Native sender implementation
    #[cfg(all(feature = "provider", feature = "native"))]
    impl PortSender for CodePortSender {
      fn get_port(&self) -> Result<&PortChannel, ProviderError> {
        if self.port.is_closed() {
          Err(ProviderError::SendChannelClosed)
        } else {
          Ok(&self.port)
        }
      }

      fn get_port_name(&self) -> &str {
        &self.port.name
      }
    }

    // WASM sender implementation
    #[cfg(all(feature = "provider", feature = "wasm"))]
    impl PortSender for CodePortSender {
      type PayloadType = u32;
      fn get_name(&self) -> String {
        "code".to_string()
      }
      fn get_id(&self) -> u32 {
        self.id
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", feature = "native"))]
    pub fn get_outputs() -> (OutputPorts, TransportStream) {
      let mut outputs = OutputPorts::default();
      let mut ports = vec![&mut outputs.code.port];
      let stream = PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[cfg(all(feature = "provider", feature = "wasm"))]
    fn get_outputs(id: u32) -> OutputPorts {
      OutputPorts {
        code: CodePortSender { id },
      }
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl Outputs {
      pub async fn code(&mut self) -> Result<PortOutput<u32>, ProviderError> {
        let packets = self.packets.drain_port("code").await;
        Ok(PortOutput::new("code".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl Outputs {
      pub fn code(&mut self) -> Result<PortOutput, ComponentError> {
        let packets = self.packets.drain_port("code")?;
        Ok(PortOutput::new("code".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl From<ProviderOutput> for Outputs {
      fn from(packets: ProviderOutput) -> Self {
        Self { packets }
      }
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl From<ProviderOutput> for Outputs {
      fn from(output: ProviderOutput) -> Self {
        Self { packets: output }
      }
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl From<BoxedTransportStream> for Outputs {
      fn from(stream: BoxedTransportStream) -> Self {
        Self {
          packets: ProviderOutput::new(stream),
        }
      }
    }
  }

  pub mod __multi__ {
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    use crate::components::__multi__ as implementation;

    #[derive(Default)]
    pub struct Component {}

    impl WapcComponent for Component {
      fn execute(&self, payload: &IncomingPayload) -> JobResult {
        let outputs = get_outputs(payload.id());
        let inputs = populate_inputs(payload)?;
        implementation::job(inputs, outputs)
      }
    }

    fn populate_inputs(payload: &IncomingPayload) -> Result<Vec<ComponentInputs>, WasmError> {
      Ok(deserialize(payload.get("inputs")?)?)
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub enum ComponentInputs {
      Main(super::main::Inputs),
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub enum ComponentOutputs {
      Main(super::main::Outputs),
    }

    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(all(feature = "provider", any(feature = "native", feature = "wasm")))]
    pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("result".to_owned(), TypeSignature::Bool);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
    #[cfg(feature = "provider")]
    pub struct OutputPorts {
      pub result: ResultPortSender,
    }

    // Definition and implementation of each port's sender.
    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct ResultPortSender {
      #[cfg(feature = "native")]
      port: PortChannel,
      #[cfg(feature = "wasm")]
      id: u32,
    }

    #[cfg(all(feature = "provider", feature = "native"))]
    impl Default for ResultPortSender {
      fn default() -> Self {
        Self {
          port: PortChannel::new("result"),
        }
      }
    }

    // Native sender implementation
    #[cfg(all(feature = "provider", feature = "native"))]
    impl PortSender for ResultPortSender {
      fn get_port(&self) -> Result<&PortChannel, ProviderError> {
        if self.port.is_closed() {
          Err(ProviderError::SendChannelClosed)
        } else {
          Ok(&self.port)
        }
      }

      fn get_port_name(&self) -> &str {
        &self.port.name
      }
    }

    // WASM sender implementation
    #[cfg(all(feature = "provider", feature = "wasm"))]
    impl PortSender for ResultPortSender {
      type PayloadType = bool;
      fn get_name(&self) -> String {
        "result".to_string()
      }
      fn get_id(&self) -> u32 {
        self.id
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", feature = "native"))]
    pub fn get_outputs() -> (OutputPorts, TransportStream) {
      let mut outputs = OutputPorts::default();
      let mut ports = vec![&mut outputs.result.port];
      let stream = PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[cfg(all(feature = "provider", feature = "wasm"))]
    fn get_outputs(id: u32) -> OutputPorts {
      OutputPorts {
        result: ResultPortSender { id },
      }
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl Outputs {
      pub async fn result(&mut self) -> Result<PortOutput<bool>, ProviderError> {
        let packets = self.packets.drain_port("result").await;
        Ok(PortOutput::new("result".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl Outputs {
      pub fn result(&mut self) -> Result<PortOutput, ComponentError> {
        let packets = self.packets.drain_port("result")?;
        Ok(PortOutput::new("result".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl From<ProviderOutput> for Outputs {
      fn from(packets: ProviderOutput) -> Self {
        Self { packets }
      }
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl From<ProviderOutput> for Outputs {
      fn from(output: ProviderOutput) -> Self {
        Self { packets: output }
      }
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl From<BoxedTransportStream> for Outputs {
      fn from(stream: BoxedTransportStream) -> Self {
        Self {
          packets: ProviderOutput::new(stream),
        }
      }
    }
  }
}
