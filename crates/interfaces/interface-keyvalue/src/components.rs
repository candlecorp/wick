/**********************************************
***** This file is generated, do not edit *****
***********************************************/

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

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      keys: payload.consume("keys")?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "keys")]
    pub keys: Vec<String>,
  }

  #[cfg(any(feature = "native", feature = "wasm"))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("keys".to_owned(), MessageTransport::success(&inputs.keys));

      map
    }
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
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
  #[derive(Debug, Default)]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub num: NumPortSender,
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("num".to_owned(), TypeSignature::U32);
    map
  }

  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct NumPortSender {
    port: PortChannel,
  }

  #[cfg(feature = "provider")]
  impl Default for NumPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("num"),
      }
    }
  }

  #[cfg(feature = "provider")]
  impl PortSender for NumPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  #[cfg(feature = "provider")]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.num.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn num(&mut self) -> Result<PortOutput<u32>, ProviderError> {
      let packets = self.packets.take("num").await;
      Ok(PortOutput::new("num".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {}

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
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

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
  }

  #[cfg(any(feature = "native", feature = "wasm"))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key".to_owned(), MessageTransport::success(&inputs.key));

      map
    }
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map
  }
  #[derive(Debug, Default)]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub exists: ExistsPortSender,
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("exists".to_owned(), TypeSignature::Bool);
    map
  }

  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct ExistsPortSender {
    port: PortChannel,
  }

  #[cfg(feature = "provider")]
  impl Default for ExistsPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("exists"),
      }
    }
  }

  #[cfg(feature = "provider")]
  impl PortSender for ExistsPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  #[cfg(feature = "provider")]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.exists.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn exists(&mut self) -> Result<PortOutput<bool>, ProviderError> {
      let packets = self.packets.take("exists").await;
      Ok(PortOutput::new("exists".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {}

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
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

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
  }

  #[cfg(any(feature = "native", feature = "wasm"))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key".to_owned(), MessageTransport::success(&inputs.key));

      map
    }
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map
  }
  #[derive(Debug, Default)]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub value: ValuePortSender,
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("value".to_owned(), TypeSignature::String);
    map
  }

  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct ValuePortSender {
    port: PortChannel,
  }

  #[cfg(feature = "provider")]
  impl Default for ValuePortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("value"),
      }
    }
  }

  #[cfg(feature = "provider")]
  impl PortSender for ValuePortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  #[cfg(feature = "provider")]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.value.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn value(&mut self) -> Result<PortOutput<String>, ProviderError> {
      let packets = self.packets.take("value").await;
      Ok(PortOutput::new("value".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {}

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
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

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
      value: payload.consume("value")?,
      expires: payload.consume("expires")?,
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

  #[cfg(any(feature = "native", feature = "wasm"))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key".to_owned(), MessageTransport::success(&inputs.key));

      map.insert("value".to_owned(), MessageTransport::success(&inputs.value));

      map.insert(
        "expires".to_owned(),
        MessageTransport::success(&inputs.expires),
      );

      map
    }
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map.insert("value".to_owned(), TypeSignature::String);
    map.insert("expires".to_owned(), TypeSignature::U32);
    map
  }
  #[derive(Debug, Default)]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub result: ResultPortSender,
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("result".to_owned(), TypeSignature::Bool);
    map
  }

  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct ResultPortSender {
    port: PortChannel,
  }

  #[cfg(feature = "provider")]
  impl Default for ResultPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("result"),
      }
    }
  }

  #[cfg(feature = "provider")]
  impl PortSender for ResultPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  #[cfg(feature = "provider")]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.result.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn result(&mut self) -> Result<PortOutput<bool>, ProviderError> {
      let packets = self.packets.take("result").await;
      Ok(PortOutput::new("result".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {}

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
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

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
      values: payload.consume("values")?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "values")]
    pub values: Vec<String>,
  }

  #[cfg(any(feature = "native", feature = "wasm"))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key".to_owned(), MessageTransport::success(&inputs.key));

      map.insert(
        "values".to_owned(),
        MessageTransport::success(&inputs.values),
      );

      map
    }
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
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
  #[derive(Debug, Default)]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub length: LengthPortSender,
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("length".to_owned(), TypeSignature::U32);
    map
  }

  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct LengthPortSender {
    port: PortChannel,
  }

  #[cfg(feature = "provider")]
  impl Default for LengthPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("length"),
      }
    }
  }

  #[cfg(feature = "provider")]
  impl PortSender for LengthPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  #[cfg(feature = "provider")]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.length.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn length(&mut self) -> Result<PortOutput<u32>, ProviderError> {
      let packets = self.packets.take("length").await;
      Ok(PortOutput::new("length".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {}

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
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

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
      start: payload.consume("start")?,
      end: payload.consume("end")?,
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

  #[cfg(any(feature = "native", feature = "wasm"))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key".to_owned(), MessageTransport::success(&inputs.key));

      map.insert("start".to_owned(), MessageTransport::success(&inputs.start));

      map.insert("end".to_owned(), MessageTransport::success(&inputs.end));

      map
    }
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map.insert("start".to_owned(), TypeSignature::I32);
    map.insert("end".to_owned(), TypeSignature::I32);
    map
  }
  #[derive(Debug, Default)]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub values: ValuesPortSender,
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
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

  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct ValuesPortSender {
    port: PortChannel,
  }

  #[cfg(feature = "provider")]
  impl Default for ValuesPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("values"),
      }
    }
  }

  #[cfg(feature = "provider")]
  impl PortSender for ValuesPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  #[cfg(feature = "provider")]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.values.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn values(&mut self) -> Result<PortOutput<Vec<String>>, ProviderError> {
      let packets = self.packets.take("values").await;
      Ok(PortOutput::new("values".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {}

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
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

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
      value: payload.consume("value")?,
      num: payload.consume("num")?,
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

  #[cfg(any(feature = "native", feature = "wasm"))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key".to_owned(), MessageTransport::success(&inputs.key));

      map.insert("value".to_owned(), MessageTransport::success(&inputs.value));

      map.insert("num".to_owned(), MessageTransport::success(&inputs.num));

      map
    }
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map.insert("value".to_owned(), TypeSignature::String);
    map.insert("num".to_owned(), TypeSignature::U32);
    map
  }
  #[derive(Debug, Default)]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub num: NumPortSender,
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("num".to_owned(), TypeSignature::U32);
    map
  }

  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct NumPortSender {
    port: PortChannel,
  }

  #[cfg(feature = "provider")]
  impl Default for NumPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("num"),
      }
    }
  }

  #[cfg(feature = "provider")]
  impl PortSender for NumPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  #[cfg(feature = "provider")]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.num.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn num(&mut self) -> Result<PortOutput<u32>, ProviderError> {
      let packets = self.packets.take("num").await;
      Ok(PortOutput::new("num".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {}

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
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

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
      values: payload.consume("values")?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "values")]
    pub values: Vec<String>,
  }

  #[cfg(any(feature = "native", feature = "wasm"))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key".to_owned(), MessageTransport::success(&inputs.key));

      map.insert(
        "values".to_owned(),
        MessageTransport::success(&inputs.values),
      );

      map
    }
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
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
  #[derive(Debug, Default)]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub length: LengthPortSender,
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("length".to_owned(), TypeSignature::U32);
    map
  }

  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct LengthPortSender {
    port: PortChannel,
  }

  #[cfg(feature = "provider")]
  impl Default for LengthPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("length"),
      }
    }
  }

  #[cfg(feature = "provider")]
  impl PortSender for LengthPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  #[cfg(feature = "provider")]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.length.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn length(&mut self) -> Result<PortOutput<u32>, ProviderError> {
      let packets = self.packets.take("length").await;
      Ok(PortOutput::new("length".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {}

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
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

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
      member: payload.consume("member")?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "member")]
    pub member: String,
  }

  #[cfg(any(feature = "native", feature = "wasm"))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key".to_owned(), MessageTransport::success(&inputs.key));

      map.insert(
        "member".to_owned(),
        MessageTransport::success(&inputs.member),
      );

      map
    }
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map.insert("member".to_owned(), TypeSignature::String);
    map
  }
  #[derive(Debug, Default)]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub exists: ExistsPortSender,
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("exists".to_owned(), TypeSignature::Bool);
    map
  }

  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct ExistsPortSender {
    port: PortChannel,
  }

  #[cfg(feature = "provider")]
  impl Default for ExistsPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("exists"),
      }
    }
  }

  #[cfg(feature = "provider")]
  impl PortSender for ExistsPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  #[cfg(feature = "provider")]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.exists.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn exists(&mut self) -> Result<PortOutput<bool>, ProviderError> {
      let packets = self.packets.take("exists").await;
      Ok(PortOutput::new("exists".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {}

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
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

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
  }

  #[cfg(any(feature = "native", feature = "wasm"))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key".to_owned(), MessageTransport::success(&inputs.key));

      map
    }
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map
  }
  #[derive(Debug, Default)]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub values: ValuesPortSender,
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
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

  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct ValuesPortSender {
    port: PortChannel,
  }

  #[cfg(feature = "provider")]
  impl Default for ValuesPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("values"),
      }
    }
  }

  #[cfg(feature = "provider")]
  impl PortSender for ValuesPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  #[cfg(feature = "provider")]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.values.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn values(&mut self) -> Result<PortOutput<Vec<String>>, ProviderError> {
      let packets = self.packets.take("values").await;
      Ok(PortOutput::new("values".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {}

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
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

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
      values: payload.consume("values")?,
    })
  }

  #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "values")]
    pub values: Vec<String>,
  }

  #[cfg(any(feature = "native", feature = "wasm"))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key".to_owned(), MessageTransport::success(&inputs.key));

      map.insert(
        "values".to_owned(),
        MessageTransport::success(&inputs.values),
      );

      map
    }
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
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
  #[derive(Debug, Default)]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub num: NumPortSender,
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("num".to_owned(), TypeSignature::U32);
    map
  }

  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct NumPortSender {
    port: PortChannel,
  }

  #[cfg(feature = "provider")]
  impl Default for NumPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("num"),
      }
    }
  }

  #[cfg(feature = "provider")]
  impl PortSender for NumPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  #[cfg(feature = "provider")]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.num.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn num(&mut self) -> Result<PortOutput<u32>, ProviderError> {
      let packets = self.packets.take("num").await;
      Ok(PortOutput::new("num".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {}

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
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

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      key: payload.consume("key")?,
      cursor: payload.consume("cursor")?,
      count: payload.consume("count")?,
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

  #[cfg(any(feature = "native", feature = "wasm"))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("key".to_owned(), MessageTransport::success(&inputs.key));

      map.insert(
        "cursor".to_owned(),
        MessageTransport::success(&inputs.cursor),
      );

      map.insert("count".to_owned(), MessageTransport::success(&inputs.count));

      map
    }
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_owned(), TypeSignature::String);
    map.insert("cursor".to_owned(), TypeSignature::String);
    map.insert("count".to_owned(), TypeSignature::U32);
    map
  }
  #[derive(Debug, Default)]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub values: ValuesPortSender,
    pub cursor: CursorPortSender,
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
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

  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct ValuesPortSender {
    port: PortChannel,
  }

  #[cfg(feature = "provider")]
  impl Default for ValuesPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("values"),
      }
    }
  }

  #[cfg(feature = "provider")]
  impl PortSender for ValuesPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }
  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct CursorPortSender {
    port: PortChannel,
  }

  #[cfg(feature = "provider")]
  impl Default for CursorPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("cursor"),
      }
    }
  }

  #[cfg(feature = "provider")]
  impl PortSender for CursorPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  #[cfg(feature = "provider")]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.values.port, &mut outputs.cursor.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(all(feature = "guest"))]
  #[allow(missing_debug_implementations)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(all(feature = "native", feature = "guest"))]
  impl Outputs {
    pub async fn values(&mut self) -> Result<PortOutput<Vec<String>>, ProviderError> {
      let packets = self.packets.take("values").await;
      Ok(PortOutput::new("values".to_owned(), packets))
    }
    pub async fn cursor(&mut self) -> Result<PortOutput<String>, ProviderError> {
      let packets = self.packets.take("cursor").await;
      Ok(PortOutput::new("cursor".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {}

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
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
