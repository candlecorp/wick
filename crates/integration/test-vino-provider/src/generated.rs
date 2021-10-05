/**********************************************
***** This file is generated, do not edit *****
***********************************************/

use vino_provider::native::prelude::*;

use crate::generated;

#[derive(Debug)]
pub(crate) struct Dispatcher {}
#[async_trait]
impl Dispatch for Dispatcher {
  type Context = crate::Context;
  async fn dispatch(
    op: &str,
    context: Self::Context,
    data: TransportMap,
  ) -> Result<TransportStream, Box<NativeComponentError>> {
    use generated::*;
    let result = match op {
      "error" => error::Component::default().execute(context, data).await,
      "test-component" => {
        test_component::Component::default()
          .execute(context, data)
          .await
      }
      _ => Err(Box::new(NativeComponentError::new(format!(
        "Component not found on this provider: {}",
        op
      )))),
    }?;
    Ok(result)
  }
}

pub(crate) fn get_signature() -> ProviderSignature {
  use std::collections::HashMap;
  let mut components = HashMap::new();

  components.insert("error".to_owned(), generated::error::signature());
  components.insert(
    "test-component".to_owned(),
    generated::test_component::signature(),
  );

  ProviderSignature {
    name: "".to_owned(),
    types: StructMap::todo(),
    components: components.into(),
  }
}

pub(crate) mod error {
  #![allow(unused, unreachable_pub)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  #[cfg(feature = "native")]
  pub use vino_provider::native::prelude::*;
  #[cfg(feature = "wasm")]
  pub use vino_provider::wasm::prelude::*;

  pub(crate) fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "error".to_owned(),
      inputs: inputs_list().into(),
      outputs: outputs_list().into(),
    }
  }

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type Context = crate::Context;
    async fn execute(
      &self,
      context: Self::Context,
      data: TransportMap,
    ) -> Result<TransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
      let (outputs, stream) = get_outputs();
      let result = tokio::spawn(crate::components::error::job(inputs, outputs, context))
        .await
        .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
      }
    }
  }

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      input: payload.consume("input")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
  }

  #[cfg(any(feature = "native", feature = "wasm"))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("input".to_owned(), MessageTransport::success(&inputs.input));

      map
    }
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn inputs_list() -> HashMap<String, TypeSignature> {
    let mut map = HashMap::new();
    map.insert("input".to_owned(), TypeSignature::String);
    map
  }

  #[derive(Debug, Default)]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub output: OutputPortSender,
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn outputs_list() -> HashMap<String, TypeSignature> {
    let mut map = HashMap::new();
    map.insert("output".to_owned(), TypeSignature::String);
    map
  }

  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct OutputPortSender {
    port: PortChannel,
  }

  #[cfg(feature = "provider")]
  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("output"),
      }
    }
  }

  #[cfg(feature = "provider")]
  impl PortSender for OutputPortSender {
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
    let mut ports = vec![&mut outputs.output.port];
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
    pub async fn output(&mut self) -> Result<PortOutput<String>, ProviderError> {
      let packets = self.packets.take("output").await;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput, WasmError> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| WasmError::ResponseMissing("output".to_owned()))?;
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
  impl From<BoxedTransportStream> for Outputs {
    fn from(stream: BoxedTransportStream) -> Self {
      Self {
        packets: ProviderOutput::new(stream),
      }
    }
  }
}
pub(crate) mod test_component {
  #![allow(unused, unreachable_pub)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  #[cfg(feature = "native")]
  pub use vino_provider::native::prelude::*;
  #[cfg(feature = "wasm")]
  pub use vino_provider::wasm::prelude::*;

  pub(crate) fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "test-component".to_owned(),
      inputs: inputs_list().into(),
      outputs: outputs_list().into(),
    }
  }

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type Context = crate::Context;
    async fn execute(
      &self,
      context: Self::Context,
      data: TransportMap,
    ) -> Result<TransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
      let (outputs, stream) = get_outputs();
      let result = tokio::spawn(crate::components::test_component::job(
        inputs, outputs, context,
      ))
      .await
      .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
      }
    }
  }

  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      input: payload.consume("input")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
  }

  #[cfg(any(feature = "native", feature = "wasm"))]
  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("input".to_owned(), MessageTransport::success(&inputs.input));

      map
    }
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn inputs_list() -> HashMap<String, TypeSignature> {
    let mut map = HashMap::new();
    map.insert("input".to_owned(), TypeSignature::String);
    map
  }

  #[derive(Debug, Default)]
  #[cfg(feature = "provider")]
  pub struct OutputPorts {
    pub output: OutputPortSender,
  }

  #[must_use]
  #[cfg(any(feature = "native", feature = "wasm"))]
  pub fn outputs_list() -> HashMap<String, TypeSignature> {
    let mut map = HashMap::new();
    map.insert("output".to_owned(), TypeSignature::String);
    map
  }

  #[derive(Debug)]
  #[cfg(feature = "provider")]
  pub struct OutputPortSender {
    port: PortChannel,
  }

  #[cfg(feature = "provider")]
  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("output"),
      }
    }
  }

  #[cfg(feature = "provider")]
  impl PortSender for OutputPortSender {
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
    let mut ports = vec![&mut outputs.output.port];
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
    pub async fn output(&mut self) -> Result<PortOutput<String>, ProviderError> {
      let packets = self.packets.take("output").await;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  #[cfg(all(feature = "wasm", feature = "guest"))]
  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput, WasmError> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| WasmError::ResponseMissing("output".to_owned()))?;
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
  impl From<BoxedTransportStream> for Outputs {
    fn from(stream: BoxedTransportStream) -> Self {
      Self {
        packets: ProviderOutput::new(stream),
      }
    }
  }
}
