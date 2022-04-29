/**********************************************
***** This file is generated, do not edit *****
***********************************************/

#[cfg(all(feature = "native", not(feature = "wasm")))]
pub use vino_provider::native::prelude::*;
#[cfg(all(feature = "wasm", not(feature = "native")))]
pub use vino_provider::wasm::prelude::*;

pub mod __multi__;
pub mod add; // add

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
    let result = match op {
      "add" => self::generated::add::Component::default().execute(context, data).await,
      "__multi__" => {
        self::generated::__multi__::Component::default()
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

pub fn get_signature() -> ProviderSignature {
  let mut components = std::collections::HashMap::new();

  components.insert("add", generated::add::signature());

  ProviderSignature {
    name: Some("vino-standalone".to_owned()),
    types: std::collections::HashMap::from([]).into(),
    components: components.into(),
  }
}

pub mod types {
  // no additional types
}

pub mod generated {

  // start namespace
  // Component name : add
  pub mod add {
    use async_trait::async_trait;
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    #[cfg(feature = "provider")]
    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "add".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default, Copy, Clone, Debug)]
    pub struct Component {}

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
        let result = tokio::spawn(crate::components::add::job(inputs, outputs, context))
          .await
          .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
        }
      }
    }

    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
      Ok(Inputs {
        left: payload.consume("left")?,
        right: payload.consume("right")?,
      })
    }

    #[cfg(all(feature = "wasm", not(feature = "native")))]
    fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs, WasmError> {
      Ok(Inputs {
        left: deserialize(payload.get("left")?)?,
        right: deserialize(payload.get("right")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "left")]
      pub left: u64,
      #[serde(rename = "right")]
      pub right: u64,
    }

    #[cfg(all(feature = "guest", any(feature = "native", feature = "wasm")))]
    impl From<Inputs> for TransportMap {
      fn from(inputs: Inputs) -> TransportMap {
        let mut map = TransportMap::new();
        map.insert("left", MessageTransport::success(&inputs.left));
        map.insert("right", MessageTransport::success(&inputs.right));
        map
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", feature = "native"))]
    pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("left".to_owned(), TypeSignature::U64);
      map.insert("right".to_owned(), TypeSignature::U64);
      map
    }
    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(all(feature = "provider", any(feature = "native", feature = "wasm")))]
    pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("output".to_owned(), TypeSignature::U64);
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
      type PayloadType = u64;
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
      pub async fn output(&mut self) -> Result<PortOutput<u64>, ProviderError> {
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

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub enum ComponentInputs {
      Add(super::add::Inputs),
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub enum ComponentOutputs {
      Add(super::add::Outputs),
    }
    #[cfg(any(feature = "native"))]
    pub use vino_provider::native::prelude::*;
    #[cfg(any(feature = "wasm"))]
    pub use vino_provider::wasm::prelude::*;

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
    use async_trait::async_trait;
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    #[derive(Default, Copy, Clone, Debug)]
    pub struct Component {}

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
        let result = tokio::spawn(crate::components::__multi__::job(inputs, outputs, context))
          .await
          .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
        }
      }
    }

    pub fn populate_inputs(mut payload: TransportMap) -> Result<Vec<ComponentInputs>, TransportError> {
      payload.consume::<Vec<ComponentInputs>>("inputs")
    }
  }
}
