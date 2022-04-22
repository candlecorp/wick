/**********************************************
***** This file is generated, do not edit *****
***********************************************/

pub mod __multi__ {
  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub enum ComponentInputs {
    Delete(super::delete::Inputs),
    Exists(super::exists::Inputs),
    KeyGet(super::key_get::Inputs),
    KeySet(super::key_set::Inputs),
    ListAdd(super::list_add::Inputs),
    ListRange(super::list_range::Inputs),
    ListRemove(super::list_remove::Inputs),
    SetAdd(super::set_add::Inputs),
    SetContains(super::set_contains::Inputs),
    SetGet(super::set_get::Inputs),
    SetRemove(super::set_remove::Inputs),
    SetScan(super::set_scan::Inputs),
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub enum ComponentOutputs {
    Delete(super::delete::Outputs),
    Exists(super::exists::Outputs),
    KeyGet(super::key_get::Outputs),
    KeySet(super::key_set::Outputs),
    ListAdd(super::list_add::Outputs),
    ListRange(super::list_range::Outputs),
    ListRemove(super::list_remove::Outputs),
    SetAdd(super::set_add::Outputs),
    SetContains(super::set_contains::Outputs),
    SetGet(super::set_get::Outputs),
    SetRemove(super::set_remove::Outputs),
    SetScan(super::set_scan::Outputs),
  }
  #[cfg(any(feature = "native"))]
  pub use vino_provider::native::prelude::*;
  #[cfg(any(feature = "wasm"))]
  pub use vino_provider::wasm::prelude::*;

  #[cfg(all(feature = "native", not(feature = "wasm")))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      inputs: payload.consume("inputs")?,
    })
  }

  #[cfg(all(feature = "wasm", not(feature = "native")))]
  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
    Ok(Inputs {
      inputs: deserialize(payload.get("inputs")?)?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "inputs")]
    pub inputs: Vec<ComponentInputs>,
  }

  #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("inputs", MessageTransport::success(&inputs.inputs));
      map
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert(
      "inputs".to_owned(),
      TypeSignature::List {
        element: Box::new(TypeSignature::Internal(InternalType::ComponentInput)),
      },
    );
    map
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

pub mod delete {

  #[cfg(any(feature = "native"))]
  pub use vino_provider::native::prelude::*;
  #[cfg(any(feature = "wasm"))]
  pub use vino_provider::wasm::prelude::*;

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "delete".to_owned(),
      inputs: inputs_list().into(),
      outputs: outputs_list().into(),
    }
  }

  #[cfg(all(feature = "native", not(feature = "wasm")))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      keys: payload.consume("keys")?,
    })
  }

  #[cfg(all(feature = "wasm", not(feature = "native")))]
  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
    Ok(Inputs {
      keys: deserialize(payload.get("keys")?)?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "keys")]
    pub keys: Vec<String>,
  }

  #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("keys", MessageTransport::success(&inputs.keys));
      map
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert(
      "keys".to_owned(),
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
    map.insert("num".to_owned(), TypeSignature::U32);
    map
  }

  // A list of output ports and their associated stream sender implementations.
  #[derive(Debug)]
  #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub num: NumPortSender,
  }

  // Definition and implementation of each port's sender.
  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct NumPortSender {
    #[cfg(feature = "native")]
    port: PortChannel,
    #[cfg(feature = "wasm")]
    id: u32,
  }

  #[cfg(all(feature = "provider", feature = "native"))]
  impl Default for NumPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("num"),
      }
    }
  }

  // Native sender implementation
  #[cfg(all(feature = "provider", feature = "native"))]
  impl PortSender for NumPortSender {
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
  impl PortSender for NumPortSender {
    type PayloadType = u32;
    fn get_name(&self) -> String {
      "num".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.num.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "provider", feature = "wasm"))]
  fn get_outputs(id: u32) -> OutputPorts {
    OutputPorts {
      num: NumPortSender { id },
    }
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn num(&mut self) -> Result<PortOutput<u32>, ProviderError> {
      let packets = self.packets.drain_port("num").await;
      Ok(PortOutput::new("num".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {
    pub fn num(&mut self) -> Result<PortOutput, ComponentError> {
      let packets = self.packets.drain_port("num")?;
      Ok(PortOutput::new("num".to_owned(), packets))
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
pub mod exists {

  #[cfg(any(feature = "native"))]
  pub use vino_provider::native::prelude::*;
  #[cfg(any(feature = "wasm"))]
  pub use vino_provider::wasm::prelude::*;

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "exists".to_owned(),
      inputs: inputs_list().into(),
      outputs: outputs_list().into(),
    }
  }

  #[cfg(all(feature = "native", not(feature = "wasm")))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
    })
  }

  #[cfg(all(feature = "wasm", not(feature = "native")))]
  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
    Ok(Inputs {
      key: deserialize(payload.get("key")?)?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
  }

  #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key", MessageTransport::success(&inputs.key));
      map
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map
  }
  // A list of ports and their type signatures.
  #[must_use]
  #[cfg(all(feature = "provider", any(feature = "native", feature = "wasm")))]
  pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("exists".to_owned(), TypeSignature::Bool);
    map
  }

  // A list of output ports and their associated stream sender implementations.
  #[derive(Debug)]
  #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub exists: ExistsPortSender,
  }

  // Definition and implementation of each port's sender.
  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct ExistsPortSender {
    #[cfg(feature = "native")]
    port: PortChannel,
    #[cfg(feature = "wasm")]
    id: u32,
  }

  #[cfg(all(feature = "provider", feature = "native"))]
  impl Default for ExistsPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("exists"),
      }
    }
  }

  // Native sender implementation
  #[cfg(all(feature = "provider", feature = "native"))]
  impl PortSender for ExistsPortSender {
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
  impl PortSender for ExistsPortSender {
    type PayloadType = bool;
    fn get_name(&self) -> String {
      "exists".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.exists.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "provider", feature = "wasm"))]
  fn get_outputs(id: u32) -> OutputPorts {
    OutputPorts {
      exists: ExistsPortSender { id },
    }
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn exists(&mut self) -> Result<PortOutput<bool>, ProviderError> {
      let packets = self.packets.drain_port("exists").await;
      Ok(PortOutput::new("exists".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {
    pub fn exists(&mut self) -> Result<PortOutput, ComponentError> {
      let packets = self.packets.drain_port("exists")?;
      Ok(PortOutput::new("exists".to_owned(), packets))
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
pub mod key_get {

  #[cfg(any(feature = "native"))]
  pub use vino_provider::native::prelude::*;
  #[cfg(any(feature = "wasm"))]
  pub use vino_provider::wasm::prelude::*;

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "key-get".to_owned(),
      inputs: inputs_list().into(),
      outputs: outputs_list().into(),
    }
  }

  #[cfg(all(feature = "native", not(feature = "wasm")))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
    })
  }

  #[cfg(all(feature = "wasm", not(feature = "native")))]
  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
    Ok(Inputs {
      key: deserialize(payload.get("key")?)?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
  }

  #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key", MessageTransport::success(&inputs.key));
      map
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map
  }
  // A list of ports and their type signatures.
  #[must_use]
  #[cfg(all(feature = "provider", any(feature = "native", feature = "wasm")))]
  pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("value".to_owned(), TypeSignature::String);
    map
  }

  // A list of output ports and their associated stream sender implementations.
  #[derive(Debug)]
  #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub value: ValuePortSender,
  }

  // Definition and implementation of each port's sender.
  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct ValuePortSender {
    #[cfg(feature = "native")]
    port: PortChannel,
    #[cfg(feature = "wasm")]
    id: u32,
  }

  #[cfg(all(feature = "provider", feature = "native"))]
  impl Default for ValuePortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("value"),
      }
    }
  }

  // Native sender implementation
  #[cfg(all(feature = "provider", feature = "native"))]
  impl PortSender for ValuePortSender {
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
  impl PortSender for ValuePortSender {
    type PayloadType = String;
    fn get_name(&self) -> String {
      "value".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.value.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "provider", feature = "wasm"))]
  fn get_outputs(id: u32) -> OutputPorts {
    OutputPorts {
      value: ValuePortSender { id },
    }
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn value(&mut self) -> Result<PortOutput<String>, ProviderError> {
      let packets = self.packets.drain_port("value").await;
      Ok(PortOutput::new("value".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {
    pub fn value(&mut self) -> Result<PortOutput, ComponentError> {
      let packets = self.packets.drain_port("value")?;
      Ok(PortOutput::new("value".to_owned(), packets))
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
pub mod key_set {

  #[cfg(any(feature = "native"))]
  pub use vino_provider::native::prelude::*;
  #[cfg(any(feature = "wasm"))]
  pub use vino_provider::wasm::prelude::*;

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "key-set".to_owned(),
      inputs: inputs_list().into(),
      outputs: outputs_list().into(),
    }
  }

  #[cfg(all(feature = "native", not(feature = "wasm")))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
      value: payload.consume("value")?,
      expires: payload.consume("expires")?,
    })
  }

  #[cfg(all(feature = "wasm", not(feature = "native")))]
  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
    Ok(Inputs {
      key: deserialize(payload.get("key")?)?,
      value: deserialize(payload.get("value")?)?,
      expires: deserialize(payload.get("expires")?)?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "value")]
    pub value: String,
    #[serde(rename = "expires")]
    pub expires: u32,
  }

  #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key", MessageTransport::success(&inputs.key));
      map.insert("value", MessageTransport::success(&inputs.value));
      map.insert("expires", MessageTransport::success(&inputs.expires));
      map
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map.insert("value".to_owned(), TypeSignature::String);
    map.insert("expires".to_owned(), TypeSignature::U32);
    map
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
pub mod list_add {

  #[cfg(any(feature = "native"))]
  pub use vino_provider::native::prelude::*;
  #[cfg(any(feature = "wasm"))]
  pub use vino_provider::wasm::prelude::*;

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "list-add".to_owned(),
      inputs: inputs_list().into(),
      outputs: outputs_list().into(),
    }
  }

  #[cfg(all(feature = "native", not(feature = "wasm")))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
      values: payload.consume("values")?,
    })
  }

  #[cfg(all(feature = "wasm", not(feature = "native")))]
  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
    Ok(Inputs {
      key: deserialize(payload.get("key")?)?,
      values: deserialize(payload.get("values")?)?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "values")]
    pub values: Vec<String>,
  }

  #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key", MessageTransport::success(&inputs.key));
      map.insert("values", MessageTransport::success(&inputs.values));
      map
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map.insert(
      "values".to_owned(),
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
    map.insert("length".to_owned(), TypeSignature::U32);
    map
  }

  // A list of output ports and their associated stream sender implementations.
  #[derive(Debug)]
  #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub length: LengthPortSender,
  }

  // Definition and implementation of each port's sender.
  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct LengthPortSender {
    #[cfg(feature = "native")]
    port: PortChannel,
    #[cfg(feature = "wasm")]
    id: u32,
  }

  #[cfg(all(feature = "provider", feature = "native"))]
  impl Default for LengthPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("length"),
      }
    }
  }

  // Native sender implementation
  #[cfg(all(feature = "provider", feature = "native"))]
  impl PortSender for LengthPortSender {
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
  impl PortSender for LengthPortSender {
    type PayloadType = u32;
    fn get_name(&self) -> String {
      "length".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.length.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "provider", feature = "wasm"))]
  fn get_outputs(id: u32) -> OutputPorts {
    OutputPorts {
      length: LengthPortSender { id },
    }
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn length(&mut self) -> Result<PortOutput<u32>, ProviderError> {
      let packets = self.packets.drain_port("length").await;
      Ok(PortOutput::new("length".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {
    pub fn length(&mut self) -> Result<PortOutput, ComponentError> {
      let packets = self.packets.drain_port("length")?;
      Ok(PortOutput::new("length".to_owned(), packets))
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
pub mod list_range {

  #[cfg(any(feature = "native"))]
  pub use vino_provider::native::prelude::*;
  #[cfg(any(feature = "wasm"))]
  pub use vino_provider::wasm::prelude::*;

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "list-range".to_owned(),
      inputs: inputs_list().into(),
      outputs: outputs_list().into(),
    }
  }

  #[cfg(all(feature = "native", not(feature = "wasm")))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
      start: payload.consume("start")?,
      end: payload.consume("end")?,
    })
  }

  #[cfg(all(feature = "wasm", not(feature = "native")))]
  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
    Ok(Inputs {
      key: deserialize(payload.get("key")?)?,
      start: deserialize(payload.get("start")?)?,
      end: deserialize(payload.get("end")?)?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "start")]
    pub start: i32,
    #[serde(rename = "end")]
    pub end: i32,
  }

  #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key", MessageTransport::success(&inputs.key));
      map.insert("start", MessageTransport::success(&inputs.start));
      map.insert("end", MessageTransport::success(&inputs.end));
      map
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map.insert("start".to_owned(), TypeSignature::I32);
    map.insert("end".to_owned(), TypeSignature::I32);
    map
  }
  // A list of ports and their type signatures.
  #[must_use]
  #[cfg(all(feature = "provider", any(feature = "native", feature = "wasm")))]
  pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert(
      "values".to_owned(),
      TypeSignature::List {
        element: Box::new(TypeSignature::String),
      },
    );
    map
  }

  // A list of output ports and their associated stream sender implementations.
  #[derive(Debug)]
  #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub values: ValuesPortSender,
  }

  // Definition and implementation of each port's sender.
  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct ValuesPortSender {
    #[cfg(feature = "native")]
    port: PortChannel,
    #[cfg(feature = "wasm")]
    id: u32,
  }

  #[cfg(all(feature = "provider", feature = "native"))]
  impl Default for ValuesPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("values"),
      }
    }
  }

  // Native sender implementation
  #[cfg(all(feature = "provider", feature = "native"))]
  impl PortSender for ValuesPortSender {
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
  impl PortSender for ValuesPortSender {
    type PayloadType = Vec<String>;
    fn get_name(&self) -> String {
      "values".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.values.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "provider", feature = "wasm"))]
  fn get_outputs(id: u32) -> OutputPorts {
    OutputPorts {
      values: ValuesPortSender { id },
    }
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn values(&mut self) -> Result<PortOutput<Vec<String>>, ProviderError> {
      let packets = self.packets.drain_port("values").await;
      Ok(PortOutput::new("values".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {
    pub fn values(&mut self) -> Result<PortOutput, ComponentError> {
      let packets = self.packets.drain_port("values")?;
      Ok(PortOutput::new("values".to_owned(), packets))
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
pub mod list_remove {

  #[cfg(any(feature = "native"))]
  pub use vino_provider::native::prelude::*;
  #[cfg(any(feature = "wasm"))]
  pub use vino_provider::wasm::prelude::*;

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "list-remove".to_owned(),
      inputs: inputs_list().into(),
      outputs: outputs_list().into(),
    }
  }

  #[cfg(all(feature = "native", not(feature = "wasm")))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
      value: payload.consume("value")?,
      num: payload.consume("num")?,
    })
  }

  #[cfg(all(feature = "wasm", not(feature = "native")))]
  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
    Ok(Inputs {
      key: deserialize(payload.get("key")?)?,
      value: deserialize(payload.get("value")?)?,
      num: deserialize(payload.get("num")?)?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "value")]
    pub value: String,
    #[serde(rename = "num")]
    pub num: u32,
  }

  #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key", MessageTransport::success(&inputs.key));
      map.insert("value", MessageTransport::success(&inputs.value));
      map.insert("num", MessageTransport::success(&inputs.num));
      map
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map.insert("value".to_owned(), TypeSignature::String);
    map.insert("num".to_owned(), TypeSignature::U32);
    map
  }
  // A list of ports and their type signatures.
  #[must_use]
  #[cfg(all(feature = "provider", any(feature = "native", feature = "wasm")))]
  pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("num".to_owned(), TypeSignature::U32);
    map
  }

  // A list of output ports and their associated stream sender implementations.
  #[derive(Debug)]
  #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub num: NumPortSender,
  }

  // Definition and implementation of each port's sender.
  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct NumPortSender {
    #[cfg(feature = "native")]
    port: PortChannel,
    #[cfg(feature = "wasm")]
    id: u32,
  }

  #[cfg(all(feature = "provider", feature = "native"))]
  impl Default for NumPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("num"),
      }
    }
  }

  // Native sender implementation
  #[cfg(all(feature = "provider", feature = "native"))]
  impl PortSender for NumPortSender {
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
  impl PortSender for NumPortSender {
    type PayloadType = u32;
    fn get_name(&self) -> String {
      "num".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.num.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "provider", feature = "wasm"))]
  fn get_outputs(id: u32) -> OutputPorts {
    OutputPorts {
      num: NumPortSender { id },
    }
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn num(&mut self) -> Result<PortOutput<u32>, ProviderError> {
      let packets = self.packets.drain_port("num").await;
      Ok(PortOutput::new("num".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {
    pub fn num(&mut self) -> Result<PortOutput, ComponentError> {
      let packets = self.packets.drain_port("num")?;
      Ok(PortOutput::new("num".to_owned(), packets))
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
pub mod set_add {

  #[cfg(any(feature = "native"))]
  pub use vino_provider::native::prelude::*;
  #[cfg(any(feature = "wasm"))]
  pub use vino_provider::wasm::prelude::*;

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "set-add".to_owned(),
      inputs: inputs_list().into(),
      outputs: outputs_list().into(),
    }
  }

  #[cfg(all(feature = "native", not(feature = "wasm")))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
      values: payload.consume("values")?,
    })
  }

  #[cfg(all(feature = "wasm", not(feature = "native")))]
  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
    Ok(Inputs {
      key: deserialize(payload.get("key")?)?,
      values: deserialize(payload.get("values")?)?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "values")]
    pub values: Vec<String>,
  }

  #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key", MessageTransport::success(&inputs.key));
      map.insert("values", MessageTransport::success(&inputs.values));
      map
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map.insert(
      "values".to_owned(),
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
    map.insert("length".to_owned(), TypeSignature::U32);
    map
  }

  // A list of output ports and their associated stream sender implementations.
  #[derive(Debug)]
  #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub length: LengthPortSender,
  }

  // Definition and implementation of each port's sender.
  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct LengthPortSender {
    #[cfg(feature = "native")]
    port: PortChannel,
    #[cfg(feature = "wasm")]
    id: u32,
  }

  #[cfg(all(feature = "provider", feature = "native"))]
  impl Default for LengthPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("length"),
      }
    }
  }

  // Native sender implementation
  #[cfg(all(feature = "provider", feature = "native"))]
  impl PortSender for LengthPortSender {
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
  impl PortSender for LengthPortSender {
    type PayloadType = u32;
    fn get_name(&self) -> String {
      "length".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.length.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "provider", feature = "wasm"))]
  fn get_outputs(id: u32) -> OutputPorts {
    OutputPorts {
      length: LengthPortSender { id },
    }
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn length(&mut self) -> Result<PortOutput<u32>, ProviderError> {
      let packets = self.packets.drain_port("length").await;
      Ok(PortOutput::new("length".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {
    pub fn length(&mut self) -> Result<PortOutput, ComponentError> {
      let packets = self.packets.drain_port("length")?;
      Ok(PortOutput::new("length".to_owned(), packets))
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
pub mod set_contains {

  #[cfg(any(feature = "native"))]
  pub use vino_provider::native::prelude::*;
  #[cfg(any(feature = "wasm"))]
  pub use vino_provider::wasm::prelude::*;

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "set-contains".to_owned(),
      inputs: inputs_list().into(),
      outputs: outputs_list().into(),
    }
  }

  #[cfg(all(feature = "native", not(feature = "wasm")))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
      member: payload.consume("member")?,
    })
  }

  #[cfg(all(feature = "wasm", not(feature = "native")))]
  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
    Ok(Inputs {
      key: deserialize(payload.get("key")?)?,
      member: deserialize(payload.get("member")?)?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "member")]
    pub member: String,
  }

  #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key", MessageTransport::success(&inputs.key));
      map.insert("member", MessageTransport::success(&inputs.member));
      map
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map.insert("member".to_owned(), TypeSignature::String);
    map
  }
  // A list of ports and their type signatures.
  #[must_use]
  #[cfg(all(feature = "provider", any(feature = "native", feature = "wasm")))]
  pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("exists".to_owned(), TypeSignature::Bool);
    map
  }

  // A list of output ports and their associated stream sender implementations.
  #[derive(Debug)]
  #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub exists: ExistsPortSender,
  }

  // Definition and implementation of each port's sender.
  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct ExistsPortSender {
    #[cfg(feature = "native")]
    port: PortChannel,
    #[cfg(feature = "wasm")]
    id: u32,
  }

  #[cfg(all(feature = "provider", feature = "native"))]
  impl Default for ExistsPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("exists"),
      }
    }
  }

  // Native sender implementation
  #[cfg(all(feature = "provider", feature = "native"))]
  impl PortSender for ExistsPortSender {
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
  impl PortSender for ExistsPortSender {
    type PayloadType = bool;
    fn get_name(&self) -> String {
      "exists".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.exists.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "provider", feature = "wasm"))]
  fn get_outputs(id: u32) -> OutputPorts {
    OutputPorts {
      exists: ExistsPortSender { id },
    }
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn exists(&mut self) -> Result<PortOutput<bool>, ProviderError> {
      let packets = self.packets.drain_port("exists").await;
      Ok(PortOutput::new("exists".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {
    pub fn exists(&mut self) -> Result<PortOutput, ComponentError> {
      let packets = self.packets.drain_port("exists")?;
      Ok(PortOutput::new("exists".to_owned(), packets))
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
pub mod set_get {

  #[cfg(any(feature = "native"))]
  pub use vino_provider::native::prelude::*;
  #[cfg(any(feature = "wasm"))]
  pub use vino_provider::wasm::prelude::*;

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "set-get".to_owned(),
      inputs: inputs_list().into(),
      outputs: outputs_list().into(),
    }
  }

  #[cfg(all(feature = "native", not(feature = "wasm")))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
    })
  }

  #[cfg(all(feature = "wasm", not(feature = "native")))]
  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
    Ok(Inputs {
      key: deserialize(payload.get("key")?)?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
  }

  #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key", MessageTransport::success(&inputs.key));
      map
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map
  }
  // A list of ports and their type signatures.
  #[must_use]
  #[cfg(all(feature = "provider", any(feature = "native", feature = "wasm")))]
  pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert(
      "values".to_owned(),
      TypeSignature::List {
        element: Box::new(TypeSignature::String),
      },
    );
    map
  }

  // A list of output ports and their associated stream sender implementations.
  #[derive(Debug)]
  #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub values: ValuesPortSender,
  }

  // Definition and implementation of each port's sender.
  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct ValuesPortSender {
    #[cfg(feature = "native")]
    port: PortChannel,
    #[cfg(feature = "wasm")]
    id: u32,
  }

  #[cfg(all(feature = "provider", feature = "native"))]
  impl Default for ValuesPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("values"),
      }
    }
  }

  // Native sender implementation
  #[cfg(all(feature = "provider", feature = "native"))]
  impl PortSender for ValuesPortSender {
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
  impl PortSender for ValuesPortSender {
    type PayloadType = Vec<String>;
    fn get_name(&self) -> String {
      "values".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.values.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "provider", feature = "wasm"))]
  fn get_outputs(id: u32) -> OutputPorts {
    OutputPorts {
      values: ValuesPortSender { id },
    }
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn values(&mut self) -> Result<PortOutput<Vec<String>>, ProviderError> {
      let packets = self.packets.drain_port("values").await;
      Ok(PortOutput::new("values".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {
    pub fn values(&mut self) -> Result<PortOutput, ComponentError> {
      let packets = self.packets.drain_port("values")?;
      Ok(PortOutput::new("values".to_owned(), packets))
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
pub mod set_remove {

  #[cfg(any(feature = "native"))]
  pub use vino_provider::native::prelude::*;
  #[cfg(any(feature = "wasm"))]
  pub use vino_provider::wasm::prelude::*;

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "set-remove".to_owned(),
      inputs: inputs_list().into(),
      outputs: outputs_list().into(),
    }
  }

  #[cfg(all(feature = "native", not(feature = "wasm")))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
      values: payload.consume("values")?,
    })
  }

  #[cfg(all(feature = "wasm", not(feature = "native")))]
  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
    Ok(Inputs {
      key: deserialize(payload.get("key")?)?,
      values: deserialize(payload.get("values")?)?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "values")]
    pub values: Vec<String>,
  }

  #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key", MessageTransport::success(&inputs.key));
      map.insert("values", MessageTransport::success(&inputs.values));
      map
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map.insert(
      "values".to_owned(),
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
    map.insert("num".to_owned(), TypeSignature::U32);
    map
  }

  // A list of output ports and their associated stream sender implementations.
  #[derive(Debug)]
  #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub num: NumPortSender,
  }

  // Definition and implementation of each port's sender.
  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct NumPortSender {
    #[cfg(feature = "native")]
    port: PortChannel,
    #[cfg(feature = "wasm")]
    id: u32,
  }

  #[cfg(all(feature = "provider", feature = "native"))]
  impl Default for NumPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("num"),
      }
    }
  }

  // Native sender implementation
  #[cfg(all(feature = "provider", feature = "native"))]
  impl PortSender for NumPortSender {
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
  impl PortSender for NumPortSender {
    type PayloadType = u32;
    fn get_name(&self) -> String {
      "num".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.num.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "provider", feature = "wasm"))]
  fn get_outputs(id: u32) -> OutputPorts {
    OutputPorts {
      num: NumPortSender { id },
    }
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn num(&mut self) -> Result<PortOutput<u32>, ProviderError> {
      let packets = self.packets.drain_port("num").await;
      Ok(PortOutput::new("num".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {
    pub fn num(&mut self) -> Result<PortOutput, ComponentError> {
      let packets = self.packets.drain_port("num")?;
      Ok(PortOutput::new("num".to_owned(), packets))
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
pub mod set_scan {

  #[cfg(any(feature = "native"))]
  pub use vino_provider::native::prelude::*;
  #[cfg(any(feature = "wasm"))]
  pub use vino_provider::wasm::prelude::*;

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "set-scan".to_owned(),
      inputs: inputs_list().into(),
      outputs: outputs_list().into(),
    }
  }

  #[cfg(all(feature = "native", not(feature = "wasm")))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
      cursor: payload.consume("cursor")?,
      count: payload.consume("count")?,
    })
  }

  #[cfg(all(feature = "wasm", not(feature = "native")))]
  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
    Ok(Inputs {
      key: deserialize(payload.get("key")?)?,
      cursor: deserialize(payload.get("cursor")?)?,
      count: deserialize(payload.get("count")?)?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "cursor")]
    pub cursor: String,
    #[serde(rename = "count")]
    pub count: u32,
  }

  #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key", MessageTransport::success(&inputs.key));
      map.insert("cursor", MessageTransport::success(&inputs.cursor));
      map.insert("count", MessageTransport::success(&inputs.count));
      map
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map.insert("cursor".to_owned(), TypeSignature::String);
    map.insert("count".to_owned(), TypeSignature::U32);
    map
  }
  // A list of ports and their type signatures.
  #[must_use]
  #[cfg(all(feature = "provider", any(feature = "native", feature = "wasm")))]
  pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert(
      "values".to_owned(),
      TypeSignature::List {
        element: Box::new(TypeSignature::String),
      },
    );
    map.insert("cursor".to_owned(), TypeSignature::String);
    map
  }

  // A list of output ports and their associated stream sender implementations.
  #[derive(Debug)]
  #[cfg_attr(all(feature = "provider", feature = "native"), derive(Default))]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub values: ValuesPortSender,
    pub cursor: CursorPortSender,
  }

  // Definition and implementation of each port's sender.
  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct ValuesPortSender {
    #[cfg(feature = "native")]
    port: PortChannel,
    #[cfg(feature = "wasm")]
    id: u32,
  }

  #[cfg(all(feature = "provider", feature = "native"))]
  impl Default for ValuesPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("values"),
      }
    }
  }

  // Native sender implementation
  #[cfg(all(feature = "provider", feature = "native"))]
  impl PortSender for ValuesPortSender {
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
  impl PortSender for ValuesPortSender {
    type PayloadType = Vec<String>;
    fn get_name(&self) -> String {
      "values".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }
  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct CursorPortSender {
    #[cfg(feature = "native")]
    port: PortChannel,
    #[cfg(feature = "wasm")]
    id: u32,
  }

  #[cfg(all(feature = "provider", feature = "native"))]
  impl Default for CursorPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("cursor"),
      }
    }
  }

  // Native sender implementation
  #[cfg(all(feature = "provider", feature = "native"))]
  impl PortSender for CursorPortSender {
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
  impl PortSender for CursorPortSender {
    type PayloadType = String;
    fn get_name(&self) -> String {
      "cursor".to_string()
    }
    fn get_id(&self) -> u32 {
      self.id
    }
  }

  #[must_use]
  #[cfg(all(feature = "provider", feature = "native"))]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.values.port, &mut outputs.cursor.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "provider", feature = "wasm"))]
  fn get_outputs(id: u32) -> OutputPorts {
    OutputPorts {
      values: ValuesPortSender { id },
      cursor: CursorPortSender { id },
    }
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn values(&mut self) -> Result<PortOutput<Vec<String>>, ProviderError> {
      let packets = self.packets.drain_port("values").await;
      Ok(PortOutput::new("values".to_owned(), packets))
    }
    pub async fn cursor(&mut self) -> Result<PortOutput<String>, ProviderError> {
      let packets = self.packets.drain_port("cursor").await;
      Ok(PortOutput::new("cursor".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {
    pub fn values(&mut self) -> Result<PortOutput, ComponentError> {
      let packets = self.packets.drain_port("values")?;
      Ok(PortOutput::new("values".to_owned(), packets))
    }
    pub fn cursor(&mut self) -> Result<PortOutput, ComponentError> {
      let packets = self.packets.drain_port("cursor")?;
      Ok(PortOutput::new("cursor".to_owned(), packets))
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
