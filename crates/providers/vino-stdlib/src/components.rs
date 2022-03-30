/**********************************************
***** This file is generated, do not edit *****
***********************************************/

pub use vino_provider::prelude::*;

pub mod core {

  pub mod error; // core::error
  pub mod log; // core::log
  pub mod panic; // core::panic
}
pub mod math {

  pub mod add; // math::add
  pub mod subtract; // math::subtract
}
pub mod rand {

  pub mod bytes; // rand::bytes
  pub mod string; // rand::string
  pub mod uuid; // rand::uuid
}
pub mod string {

  pub mod concat; // string::concat
}
pub mod __multi__;

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
      "core::error" => {
        self::generated::core::error::Component::default()
          .execute(context, data)
          .await
      }
      "core::log" => {
        self::generated::core::log::Component::default()
          .execute(context, data)
          .await
      }
      "core::panic" => {
        self::generated::core::panic::Component::default()
          .execute(context, data)
          .await
      }
      "math::add" => {
        self::generated::math::add::Component::default()
          .execute(context, data)
          .await
      }
      "math::subtract" => {
        self::generated::math::subtract::Component::default()
          .execute(context, data)
          .await
      }
      "rand::bytes" => {
        self::generated::rand::bytes::Component::default()
          .execute(context, data)
          .await
      }
      "rand::string" => {
        self::generated::rand::string::Component::default()
          .execute(context, data)
          .await
      }
      "rand::uuid" => {
        self::generated::rand::uuid::Component::default()
          .execute(context, data)
          .await
      }
      "string::concat" => {
        self::generated::string::concat::Component::default()
          .execute(context, data)
          .await
      }
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

  components.insert("core::error", generated::core::error::signature());
  components.insert("core::log", generated::core::log::signature());
  components.insert("core::panic", generated::core::panic::signature());
  components.insert("math::add", generated::math::add::signature());
  components.insert("math::subtract", generated::math::subtract::signature());
  components.insert("rand::bytes", generated::rand::bytes::signature());
  components.insert("rand::string", generated::rand::string::signature());
  components.insert("rand::uuid", generated::rand::uuid::signature());
  components.insert("string::concat", generated::string::concat::signature());

  ProviderSignature {
    name: Some("vino-stdlib".to_owned()),
    types: std::collections::HashMap::from([]).into(),
    components: components.into(),
  }
}

pub mod types {
  // no additional types
}

pub mod generated {

  // start namespace core
  pub mod core {

    // start namespace
    // Component name : core::error
    pub mod error {
      use async_trait::async_trait;

      pub use vino_provider::prelude::*;

      pub fn signature() -> ComponentSignature {
        ComponentSignature {
          name: "core::error".to_owned(),
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
          let result = tokio::spawn(crate::components::core::error::job(inputs, outputs, context))
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

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "input")]
        pub input: String,
      }

      #[cfg(any(feature = "native", feature = "wasm"))]
      impl From<Inputs> for TransportMap {
        fn from(inputs: Inputs) -> TransportMap {
          let mut map = TransportMap::new();
          map.insert("input", MessageTransport::success(&inputs.input));

          map
        }
      }

      #[must_use]
      #[cfg(any(feature = "native", feature = "wasm"))]
      pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
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
      pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
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

        fn get_port_name(&self) -> &str {
          &self.port.name
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
    // Component name : core::log
    pub mod log {
      use async_trait::async_trait;

      pub use vino_provider::prelude::*;

      pub fn signature() -> ComponentSignature {
        ComponentSignature {
          name: "core::log".to_owned(),
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
          let result = tokio::spawn(crate::components::core::log::job(inputs, outputs, context))
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

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "input")]
        pub input: String,
      }

      #[cfg(any(feature = "native", feature = "wasm"))]
      impl From<Inputs> for TransportMap {
        fn from(inputs: Inputs) -> TransportMap {
          let mut map = TransportMap::new();
          map.insert("input", MessageTransport::success(&inputs.input));

          map
        }
      }

      #[must_use]
      #[cfg(any(feature = "native", feature = "wasm"))]
      pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
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
      pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
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

        fn get_port_name(&self) -> &str {
          &self.port.name
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
    // Component name : core::panic
    pub mod panic {
      use async_trait::async_trait;

      pub use vino_provider::prelude::*;

      pub fn signature() -> ComponentSignature {
        ComponentSignature {
          name: "core::panic".to_owned(),
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
          let result = tokio::spawn(crate::components::core::panic::job(inputs, outputs, context))
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

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "input")]
        pub input: String,
      }

      #[cfg(any(feature = "native", feature = "wasm"))]
      impl From<Inputs> for TransportMap {
        fn from(inputs: Inputs) -> TransportMap {
          let mut map = TransportMap::new();
          map.insert("input", MessageTransport::success(&inputs.input));

          map
        }
      }

      #[must_use]
      #[cfg(any(feature = "native", feature = "wasm"))]
      pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
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
      pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
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

        fn get_port_name(&self) -> &str {
          &self.port.name
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
  } // end namespace core
    // start namespace math
  pub mod math {

    // start namespace
    // Component name : math::add
    pub mod add {
      use async_trait::async_trait;

      pub use vino_provider::prelude::*;

      pub fn signature() -> ComponentSignature {
        ComponentSignature {
          name: "math::add".to_owned(),
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
          let result = tokio::spawn(crate::components::math::add::job(inputs, outputs, context))
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
          left: payload.consume("left")?,
          right: payload.consume("right")?,
        })
      }

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "left")]
        pub left: u64,
        #[serde(rename = "right")]
        pub right: u64,
      }

      #[cfg(any(feature = "native", feature = "wasm"))]
      impl From<Inputs> for TransportMap {
        fn from(inputs: Inputs) -> TransportMap {
          let mut map = TransportMap::new();
          map.insert("left", MessageTransport::success(&inputs.left));

          map.insert("right", MessageTransport::success(&inputs.right));

          map
        }
      }

      #[must_use]
      #[cfg(any(feature = "native", feature = "wasm"))]
      pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("left".to_owned(), TypeSignature::U64);
        map.insert("right".to_owned(), TypeSignature::U64);
        map
      }
      #[derive(Debug, Default)]
      #[cfg(feature = "provider")]
      pub struct OutputPorts {
        pub output: OutputPortSender,
      }

      #[must_use]
      #[cfg(any(feature = "native", feature = "wasm"))]
      pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("output".to_owned(), TypeSignature::U64);
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

        fn get_port_name(&self) -> &str {
          &self.port.name
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
        pub async fn output(&mut self) -> Result<PortOutput<u64>, ProviderError> {
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
    // Component name : math::subtract
    pub mod subtract {
      use async_trait::async_trait;

      pub use vino_provider::prelude::*;

      pub fn signature() -> ComponentSignature {
        ComponentSignature {
          name: "math::subtract".to_owned(),
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
          let result = tokio::spawn(crate::components::math::subtract::job(inputs, outputs, context))
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
          left: payload.consume("left")?,
          right: payload.consume("right")?,
        })
      }

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "left")]
        pub left: u64,
        #[serde(rename = "right")]
        pub right: u64,
      }

      #[cfg(any(feature = "native", feature = "wasm"))]
      impl From<Inputs> for TransportMap {
        fn from(inputs: Inputs) -> TransportMap {
          let mut map = TransportMap::new();
          map.insert("left", MessageTransport::success(&inputs.left));

          map.insert("right", MessageTransport::success(&inputs.right));

          map
        }
      }

      #[must_use]
      #[cfg(any(feature = "native", feature = "wasm"))]
      pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("left".to_owned(), TypeSignature::U64);
        map.insert("right".to_owned(), TypeSignature::U64);
        map
      }
      #[derive(Debug, Default)]
      #[cfg(feature = "provider")]
      pub struct OutputPorts {
        pub output: OutputPortSender,
      }

      #[must_use]
      #[cfg(any(feature = "native", feature = "wasm"))]
      pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("output".to_owned(), TypeSignature::U64);
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

        fn get_port_name(&self) -> &str {
          &self.port.name
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
        pub async fn output(&mut self) -> Result<PortOutput<u64>, ProviderError> {
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
  } // end namespace math
    // start namespace rand
  pub mod rand {

    // start namespace
    // Component name : rand::bytes
    pub mod bytes {
      use async_trait::async_trait;

      pub use vino_provider::prelude::*;

      pub fn signature() -> ComponentSignature {
        ComponentSignature {
          name: "rand::bytes".to_owned(),
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
          let result = tokio::spawn(crate::components::rand::bytes::job(inputs, outputs, context))
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
          seed: payload.consume("seed")?,
          length: payload.consume("length")?,
        })
      }

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "seed")]
        pub seed: u64,
        #[serde(rename = "length")]
        pub length: u32,
      }

      #[cfg(any(feature = "native", feature = "wasm"))]
      impl From<Inputs> for TransportMap {
        fn from(inputs: Inputs) -> TransportMap {
          let mut map = TransportMap::new();
          map.insert("seed", MessageTransport::success(&inputs.seed));

          map.insert("length", MessageTransport::success(&inputs.length));

          map
        }
      }

      #[must_use]
      #[cfg(any(feature = "native", feature = "wasm"))]
      pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("seed".to_owned(), TypeSignature::U64);
        map.insert("length".to_owned(), TypeSignature::U32);
        map
      }
      #[derive(Debug, Default)]
      #[cfg(feature = "provider")]
      pub struct OutputPorts {
        pub output: OutputPortSender,
      }

      #[must_use]
      #[cfg(any(feature = "native", feature = "wasm"))]
      pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("output".to_owned(), TypeSignature::Bytes);
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

        fn get_port_name(&self) -> &str {
          &self.port.name
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
        pub async fn output(&mut self) -> Result<PortOutput<Vec<u8>>, ProviderError> {
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
    // Component name : rand::string
    pub mod string {
      use async_trait::async_trait;

      pub use vino_provider::prelude::*;

      pub fn signature() -> ComponentSignature {
        ComponentSignature {
          name: "rand::string".to_owned(),
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
          let result = tokio::spawn(crate::components::rand::string::job(inputs, outputs, context))
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
          seed: payload.consume("seed")?,
          length: payload.consume("length")?,
        })
      }

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "seed")]
        pub seed: u64,
        #[serde(rename = "length")]
        pub length: u32,
      }

      #[cfg(any(feature = "native", feature = "wasm"))]
      impl From<Inputs> for TransportMap {
        fn from(inputs: Inputs) -> TransportMap {
          let mut map = TransportMap::new();
          map.insert("seed", MessageTransport::success(&inputs.seed));

          map.insert("length", MessageTransport::success(&inputs.length));

          map
        }
      }

      #[must_use]
      #[cfg(any(feature = "native", feature = "wasm"))]
      pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("seed".to_owned(), TypeSignature::U64);
        map.insert("length".to_owned(), TypeSignature::U32);
        map
      }
      #[derive(Debug, Default)]
      #[cfg(feature = "provider")]
      pub struct OutputPorts {
        pub output: OutputPortSender,
      }

      #[must_use]
      #[cfg(any(feature = "native", feature = "wasm"))]
      pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
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

        fn get_port_name(&self) -> &str {
          &self.port.name
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
    // Component name : rand::uuid
    pub mod uuid {
      use async_trait::async_trait;

      pub use vino_provider::prelude::*;

      pub fn signature() -> ComponentSignature {
        ComponentSignature {
          name: "rand::uuid".to_owned(),
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
          let result = tokio::spawn(crate::components::rand::uuid::job(inputs, outputs, context))
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
          seed: payload.consume("seed")?,
        })
      }

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "seed")]
        pub seed: u64,
      }

      #[cfg(any(feature = "native", feature = "wasm"))]
      impl From<Inputs> for TransportMap {
        fn from(inputs: Inputs) -> TransportMap {
          let mut map = TransportMap::new();
          map.insert("seed", MessageTransport::success(&inputs.seed));

          map
        }
      }

      #[must_use]
      #[cfg(any(feature = "native", feature = "wasm"))]
      pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("seed".to_owned(), TypeSignature::U64);
        map
      }
      #[derive(Debug, Default)]
      #[cfg(feature = "provider")]
      pub struct OutputPorts {
        pub output: OutputPortSender,
      }

      #[must_use]
      #[cfg(any(feature = "native", feature = "wasm"))]
      pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
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

        fn get_port_name(&self) -> &str {
          &self.port.name
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
  } // end namespace rand
    // start namespace string
  pub mod string {

    // start namespace
    // Component name : string::concat
    pub mod concat {
      use async_trait::async_trait;

      pub use vino_provider::prelude::*;

      pub fn signature() -> ComponentSignature {
        ComponentSignature {
          name: "string::concat".to_owned(),
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
          let result = tokio::spawn(crate::components::string::concat::job(inputs, outputs, context))
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
          left: payload.consume("left")?,
          right: payload.consume("right")?,
        })
      }

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "left")]
        pub left: String,
        #[serde(rename = "right")]
        pub right: String,
      }

      #[cfg(any(feature = "native", feature = "wasm"))]
      impl From<Inputs> for TransportMap {
        fn from(inputs: Inputs) -> TransportMap {
          let mut map = TransportMap::new();
          map.insert("left", MessageTransport::success(&inputs.left));

          map.insert("right", MessageTransport::success(&inputs.right));

          map
        }
      }

      #[must_use]
      #[cfg(any(feature = "native", feature = "wasm"))]
      pub fn inputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("left".to_owned(), TypeSignature::String);
        map.insert("right".to_owned(), TypeSignature::String);
        map
      }
      #[derive(Debug, Default)]
      #[cfg(feature = "provider")]
      pub struct OutputPorts {
        pub output: OutputPortSender,
      }

      #[must_use]
      #[cfg(any(feature = "native", feature = "wasm"))]
      pub fn outputs_list() -> std::collections::HashMap<String, TypeSignature> {
        let mut map = std::collections::HashMap::new();
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

        fn get_port_name(&self) -> &str {
          &self.port.name
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
  } // end namespace string

  pub mod __multi__ {

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub enum ComponentInputs {
      CoreError(super::core::error::Inputs),
      CoreLog(super::core::log::Inputs),
      CorePanic(super::core::panic::Inputs),
      MathAdd(super::math::add::Inputs),
      MathSubtract(super::math::subtract::Inputs),
      RandBytes(super::rand::bytes::Inputs),
      RandString(super::rand::string::Inputs),
      RandUuid(super::rand::uuid::Inputs),
      StringConcat(super::string::concat::Inputs),
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub enum ComponentOutputs {
      CoreError(super::core::error::Outputs),
      CoreLog(super::core::log::Outputs),
      CorePanic(super::core::panic::Outputs),
      MathAdd(super::math::add::Outputs),
      MathSubtract(super::math::subtract::Outputs),
      RandBytes(super::rand::bytes::Outputs),
      RandString(super::rand::string::Outputs),
      RandUuid(super::rand::uuid::Outputs),
      StringConcat(super::string::concat::Outputs),
    }
    #[cfg(any(feature = "native"))]
    pub use vino_provider::native::prelude::*;
    #[cfg(any(feature = "wasm"))]
    pub use vino_provider::wasm::prelude::*;

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

      fn get_port_name(&self) -> &str {
        &self.port.name
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
    impl Outputs {
      pub fn result(&mut self) -> Result<PortOutput, WasmError> {
        let packets = self
          .packets
          .take("result")
          .ok_or_else(|| WasmError::ResponseMissing("result".to_owned()))?;
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

    pub use vino_provider::prelude::*;

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
