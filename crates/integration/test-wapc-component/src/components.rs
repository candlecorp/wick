/**********************************************
***** This file is generated, do not edit *****
***********************************************/

#[cfg(all(feature = "native", not(feature = "wasm")))]
pub use vino_provider::native::prelude::*;
#[cfg(all(feature = "wasm", not(feature = "native")))]
pub use vino_provider::wasm::prelude::*;

pub mod __multi__;
pub mod copy; // copy
pub mod error; // error
pub mod reverse; // reverse
pub mod reverse_uppercase; // reverse-uppercase
pub mod uppercase; // uppercase
pub mod validate; // validate

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

  // start namespace
  pub mod copy {
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    use crate::components::copy as implementation;

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
        input: payload.consume("input")?,
        times: payload.consume("times")?,
      })
    }

    #[cfg(all(feature = "wasm", not(feature = "native")))]
    fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
      Ok(Inputs {
        input: deserialize(payload.get("input")?)?,
        times: deserialize(payload.get("times")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "input")]
      pub input: String,
      #[serde(rename = "times")]
      pub times: i8,
    }

    #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
    impl From<Inputs> for TransportMap {
      fn from(inputs: Inputs) -> TransportMap {
        let mut map = TransportMap::new();
        map.insert("input", MessageTransport::success(&inputs.input));
        map.insert("times", MessageTransport::success(&inputs.times));
        map
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", feature = "native"))]
    pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("input".to_owned(), TypeSignature::String);
      map.insert("times".to_owned(), TypeSignature::I8);
      map
    }
    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(all(feature = "provider", any(feature = "native", feature = "wasm")))]
    pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("output".to_owned(), TypeSignature::String);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
    #[cfg(feature = "provider")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    // Definition and implementation of each port's sender.
    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct OutputPortSender {
      #[cfg(feature = "native")]
      port: PortChannel,
      #[cfg(feature = "wasm")]
      id: u32,
    }

    #[cfg(all(feature = "provider", feature = "native"))]
    impl Default for OutputPortSender {
      fn default() -> Self {
        Self {
          port: PortChannel::new("output"),
        }
      }
    }

    // Native sender implementation
    #[cfg(all(feature = "provider", feature = "native"))]
    impl PortSender for OutputPortSender {
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
    impl PortSender for OutputPortSender {
      type PayloadType = String;
      fn get_name(&self) -> String {
        "output".to_string()
      }
      fn get_id(&self) -> u32 {
        self.id
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", feature = "native"))]
    pub fn get_outputs() -> (OutputPorts, TransportStream) {
      let mut outputs = OutputPorts::default();
      let mut ports = vec![&mut outputs.output.port];
      let stream = PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[cfg(all(feature = "provider", feature = "wasm"))]
    fn get_outputs(id: u32) -> OutputPorts {
      OutputPorts {
        output: OutputPortSender { id },
      }
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl Outputs {
      pub async fn output(&mut self) -> Result<PortOutput<String>, ProviderError> {
        let packets = self.packets.drain_port("output").await;
        Ok(PortOutput::new("output".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl Outputs {
      pub fn output(&mut self) -> Result<PortOutput, ComponentError> {
        let packets = self.packets.drain_port("output")?;
        Ok(PortOutput::new("output".to_owned(), packets))
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
  pub mod error {
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    use crate::components::error as implementation;

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
        input: payload.consume("input")?,
      })
    }

    #[cfg(all(feature = "wasm", not(feature = "native")))]
    fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
      Ok(Inputs {
        input: deserialize(payload.get("input")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "input")]
      pub input: String,
    }

    #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
    impl From<Inputs> for TransportMap {
      fn from(inputs: Inputs) -> TransportMap {
        let mut map = TransportMap::new();
        map.insert("input", MessageTransport::success(&inputs.input));
        map
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", feature = "native"))]
    pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("input".to_owned(), TypeSignature::String);
      map
    }
    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(all(feature = "provider", any(feature = "native", feature = "wasm")))]
    pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("output".to_owned(), TypeSignature::String);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
    #[cfg(feature = "provider")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    // Definition and implementation of each port's sender.
    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct OutputPortSender {
      #[cfg(feature = "native")]
      port: PortChannel,
      #[cfg(feature = "wasm")]
      id: u32,
    }

    #[cfg(all(feature = "provider", feature = "native"))]
    impl Default for OutputPortSender {
      fn default() -> Self {
        Self {
          port: PortChannel::new("output"),
        }
      }
    }

    // Native sender implementation
    #[cfg(all(feature = "provider", feature = "native"))]
    impl PortSender for OutputPortSender {
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
    impl PortSender for OutputPortSender {
      type PayloadType = String;
      fn get_name(&self) -> String {
        "output".to_string()
      }
      fn get_id(&self) -> u32 {
        self.id
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", feature = "native"))]
    pub fn get_outputs() -> (OutputPorts, TransportStream) {
      let mut outputs = OutputPorts::default();
      let mut ports = vec![&mut outputs.output.port];
      let stream = PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[cfg(all(feature = "provider", feature = "wasm"))]
    fn get_outputs(id: u32) -> OutputPorts {
      OutputPorts {
        output: OutputPortSender { id },
      }
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl Outputs {
      pub async fn output(&mut self) -> Result<PortOutput<String>, ProviderError> {
        let packets = self.packets.drain_port("output").await;
        Ok(PortOutput::new("output".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl Outputs {
      pub fn output(&mut self) -> Result<PortOutput, ComponentError> {
        let packets = self.packets.drain_port("output")?;
        Ok(PortOutput::new("output".to_owned(), packets))
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
  pub mod reverse {
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    use crate::components::reverse as implementation;

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
        input: payload.consume("input")?,
      })
    }

    #[cfg(all(feature = "wasm", not(feature = "native")))]
    fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
      Ok(Inputs {
        input: deserialize(payload.get("input")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "input")]
      pub input: String,
    }

    #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
    impl From<Inputs> for TransportMap {
      fn from(inputs: Inputs) -> TransportMap {
        let mut map = TransportMap::new();
        map.insert("input", MessageTransport::success(&inputs.input));
        map
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", feature = "native"))]
    pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("input".to_owned(), TypeSignature::String);
      map
    }
    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(all(feature = "provider", any(feature = "native", feature = "wasm")))]
    pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("output".to_owned(), TypeSignature::String);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
    #[cfg(feature = "provider")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    // Definition and implementation of each port's sender.
    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct OutputPortSender {
      #[cfg(feature = "native")]
      port: PortChannel,
      #[cfg(feature = "wasm")]
      id: u32,
    }

    #[cfg(all(feature = "provider", feature = "native"))]
    impl Default for OutputPortSender {
      fn default() -> Self {
        Self {
          port: PortChannel::new("output"),
        }
      }
    }

    // Native sender implementation
    #[cfg(all(feature = "provider", feature = "native"))]
    impl PortSender for OutputPortSender {
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
    impl PortSender for OutputPortSender {
      type PayloadType = String;
      fn get_name(&self) -> String {
        "output".to_string()
      }
      fn get_id(&self) -> u32 {
        self.id
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", feature = "native"))]
    pub fn get_outputs() -> (OutputPorts, TransportStream) {
      let mut outputs = OutputPorts::default();
      let mut ports = vec![&mut outputs.output.port];
      let stream = PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[cfg(all(feature = "provider", feature = "wasm"))]
    fn get_outputs(id: u32) -> OutputPorts {
      OutputPorts {
        output: OutputPortSender { id },
      }
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl Outputs {
      pub async fn output(&mut self) -> Result<PortOutput<String>, ProviderError> {
        let packets = self.packets.drain_port("output").await;
        Ok(PortOutput::new("output".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl Outputs {
      pub fn output(&mut self) -> Result<PortOutput, ComponentError> {
        let packets = self.packets.drain_port("output")?;
        Ok(PortOutput::new("output".to_owned(), packets))
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
  pub mod reverse_uppercase {
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    use crate::components::reverse_uppercase as implementation;

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
        input: payload.consume("input")?,
        link: payload.consume("link")?,
      })
    }

    #[cfg(all(feature = "wasm", not(feature = "native")))]
    fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
      Ok(Inputs {
        input: deserialize(payload.get("input")?)?,
        link: deserialize(payload.get("link")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "input")]
      pub input: String,
      #[serde(rename = "link")]
      pub link: ProviderLink,
    }

    #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
    impl From<Inputs> for TransportMap {
      fn from(inputs: Inputs) -> TransportMap {
        let mut map = TransportMap::new();
        map.insert("input", MessageTransport::success(&inputs.input));
        map.insert("link", MessageTransport::success(&inputs.link));
        map
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", feature = "native"))]
    pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("input".to_owned(), TypeSignature::String);
      map.insert(
        "link".to_owned(),
        Link {
          provider: Some("".to_owned()),
        },
      );
      map
    }
    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(all(feature = "provider", any(feature = "native", feature = "wasm")))]
    pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("output".to_owned(), TypeSignature::String);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
    #[cfg(feature = "provider")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    // Definition and implementation of each port's sender.
    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct OutputPortSender {
      #[cfg(feature = "native")]
      port: PortChannel,
      #[cfg(feature = "wasm")]
      id: u32,
    }

    #[cfg(all(feature = "provider", feature = "native"))]
    impl Default for OutputPortSender {
      fn default() -> Self {
        Self {
          port: PortChannel::new("output"),
        }
      }
    }

    // Native sender implementation
    #[cfg(all(feature = "provider", feature = "native"))]
    impl PortSender for OutputPortSender {
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
    impl PortSender for OutputPortSender {
      type PayloadType = String;
      fn get_name(&self) -> String {
        "output".to_string()
      }
      fn get_id(&self) -> u32 {
        self.id
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", feature = "native"))]
    pub fn get_outputs() -> (OutputPorts, TransportStream) {
      let mut outputs = OutputPorts::default();
      let mut ports = vec![&mut outputs.output.port];
      let stream = PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[cfg(all(feature = "provider", feature = "wasm"))]
    fn get_outputs(id: u32) -> OutputPorts {
      OutputPorts {
        output: OutputPortSender { id },
      }
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl Outputs {
      pub async fn output(&mut self) -> Result<PortOutput<String>, ProviderError> {
        let packets = self.packets.drain_port("output").await;
        Ok(PortOutput::new("output".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl Outputs {
      pub fn output(&mut self) -> Result<PortOutput, ComponentError> {
        let packets = self.packets.drain_port("output")?;
        Ok(PortOutput::new("output".to_owned(), packets))
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
  pub mod uppercase {
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    use crate::components::uppercase as implementation;

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
        input: payload.consume("input")?,
      })
    }

    #[cfg(all(feature = "wasm", not(feature = "native")))]
    fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
      Ok(Inputs {
        input: deserialize(payload.get("input")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "input")]
      pub input: String,
    }

    #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
    impl From<Inputs> for TransportMap {
      fn from(inputs: Inputs) -> TransportMap {
        let mut map = TransportMap::new();
        map.insert("input", MessageTransport::success(&inputs.input));
        map
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", feature = "native"))]
    pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("input".to_owned(), TypeSignature::String);
      map
    }
    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(all(feature = "provider", any(feature = "native", feature = "wasm")))]
    pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("output".to_owned(), TypeSignature::String);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
    #[cfg(feature = "provider")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    // Definition and implementation of each port's sender.
    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct OutputPortSender {
      #[cfg(feature = "native")]
      port: PortChannel,
      #[cfg(feature = "wasm")]
      id: u32,
    }

    #[cfg(all(feature = "provider", feature = "native"))]
    impl Default for OutputPortSender {
      fn default() -> Self {
        Self {
          port: PortChannel::new("output"),
        }
      }
    }

    // Native sender implementation
    #[cfg(all(feature = "provider", feature = "native"))]
    impl PortSender for OutputPortSender {
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
    impl PortSender for OutputPortSender {
      type PayloadType = String;
      fn get_name(&self) -> String {
        "output".to_string()
      }
      fn get_id(&self) -> u32 {
        self.id
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", feature = "native"))]
    pub fn get_outputs() -> (OutputPorts, TransportStream) {
      let mut outputs = OutputPorts::default();
      let mut ports = vec![&mut outputs.output.port];
      let stream = PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[cfg(all(feature = "provider", feature = "wasm"))]
    fn get_outputs(id: u32) -> OutputPorts {
      OutputPorts {
        output: OutputPortSender { id },
      }
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl Outputs {
      pub async fn output(&mut self) -> Result<PortOutput<String>, ProviderError> {
        let packets = self.packets.drain_port("output").await;
        Ok(PortOutput::new("output".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl Outputs {
      pub fn output(&mut self) -> Result<PortOutput, ComponentError> {
        let packets = self.packets.drain_port("output")?;
        Ok(PortOutput::new("output".to_owned(), packets))
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
  pub mod validate {
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    use crate::components::validate as implementation;

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
        input: payload.consume("input")?,
      })
    }

    #[cfg(all(feature = "wasm", not(feature = "native")))]
    fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
      Ok(Inputs {
        input: deserialize(payload.get("input")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "input")]
      pub input: String,
    }

    #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
    impl From<Inputs> for TransportMap {
      fn from(inputs: Inputs) -> TransportMap {
        let mut map = TransportMap::new();
        map.insert("input", MessageTransport::success(&inputs.input));
        map
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", feature = "native"))]
    pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("input".to_owned(), TypeSignature::String);
      map
    }
    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(all(feature = "provider", any(feature = "native", feature = "wasm")))]
    pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("output".to_owned(), TypeSignature::String);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
    #[cfg(feature = "provider")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    // Definition and implementation of each port's sender.
    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct OutputPortSender {
      #[cfg(feature = "native")]
      port: PortChannel,
      #[cfg(feature = "wasm")]
      id: u32,
    }

    #[cfg(all(feature = "provider", feature = "native"))]
    impl Default for OutputPortSender {
      fn default() -> Self {
        Self {
          port: PortChannel::new("output"),
        }
      }
    }

    // Native sender implementation
    #[cfg(all(feature = "provider", feature = "native"))]
    impl PortSender for OutputPortSender {
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
    impl PortSender for OutputPortSender {
      type PayloadType = String;
      fn get_name(&self) -> String {
        "output".to_string()
      }
      fn get_id(&self) -> u32 {
        self.id
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", feature = "native"))]
    pub fn get_outputs() -> (OutputPorts, TransportStream) {
      let mut outputs = OutputPorts::default();
      let mut ports = vec![&mut outputs.output.port];
      let stream = PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[cfg(all(feature = "provider", feature = "wasm"))]
    fn get_outputs(id: u32) -> OutputPorts {
      OutputPorts {
        output: OutputPortSender { id },
      }
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl Outputs {
      pub async fn output(&mut self) -> Result<PortOutput<String>, ProviderError> {
        let packets = self.packets.drain_port("output").await;
        Ok(PortOutput::new("output".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl Outputs {
      pub fn output(&mut self) -> Result<PortOutput, ComponentError> {
        let packets = self.packets.drain_port("output")?;
        Ok(PortOutput::new("output".to_owned(), packets))
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
