/**********************************************
***** This file is generated, do not edit *****
***********************************************/
#![allow(
  unused_qualifications,
  unused_imports,
  missing_copy_implementations,
  unused_qualifications
)]

use wasmflow_sdk::v1::ephemeral::BatchedJobExecutor;

#[cfg(all(target_arch = "wasm32"))]
type CallResult = wasmflow_sdk::v1::BoxedFuture<Result<Vec<u8>, wasmflow_sdk::v1::BoxedError>>;

#[cfg(all(target_arch = "wasm32"))]
#[allow(unsafe_code)]
#[no_mangle]
pub(crate) extern "C" fn wapc_init() {
  wasmflow_sdk::v1::wasm::runtime::register_dispatcher(Box::new(ComponentDispatcher::default()));
}

pub mod __batch__;
pub mod error; // error
pub mod reverse; // reverse
pub mod reverse_uppercase; // reverse-uppercase
pub mod scratch; // scratch
pub mod uppercase; // uppercase
pub mod validate; // validate

#[allow(unused)]
static ALL_COMPONENTS: &[&str] = &[
  "error",
  "reverse",
  "reverse-uppercase",
  "scratch",
  "uppercase",
  "validate",
];

#[derive(Default, Copy, Clone)]
#[allow(missing_debug_implementations)]
pub struct ComponentDispatcher {}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::too_many_lines)]
impl wasmflow_sdk::v1::ephemeral::WasmDispatcher for ComponentDispatcher {
  fn dispatch(&self, op: &'static str, payload: &'static [u8]) -> CallResult {
    Box::pin(async move {
      let (mut stream, id) = match op {
        "error" => {
          crate::components::generated::error::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?)
            .await
        }
        "reverse" => {
          crate::components::generated::reverse::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?)
            .await
        }
        "reverse-uppercase" => {
          crate::components::generated::reverse_uppercase::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?)
            .await
        }
        "scratch" => {
          crate::components::generated::scratch::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?)
            .await
        }
        "uppercase" => {
          crate::components::generated::uppercase::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?)
            .await
        }
        "validate" => {
          crate::components::generated::validate::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?)
            .await
        }
        _ => Err(wasmflow_sdk::v1::error::Error::ComponentNotFound(op.to_owned(), ALL_COMPONENTS.join(", ")).into()),
      }?;
      while let Some(next) = wasmflow_sdk::v1::StreamExt::next(&mut stream).await {
        wasmflow_sdk::v1::wasm::port_send(&next.port, id, next.payload)?;
      }

      Ok(Vec::new())
    })
  }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_lines)]
impl wasmflow_sdk::v1::ephemeral::NativeDispatcher for ComponentDispatcher {
  fn dispatch(
    &self,
    invocation: wasmflow_sdk::v1::Invocation,
  ) -> wasmflow_sdk::v1::BoxedFuture<Result<wasmflow_sdk::v1::PacketStream, wasmflow_sdk::v1::BoxedError>> {
    Box::pin(async move {
      let (stream, _id) = match invocation.target.name() {
        "error" => {
          crate::components::generated::error::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?)
            .await
        }
        "reverse" => {
          crate::components::generated::reverse::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?)
            .await
        }
        "reverse-uppercase" => {
          crate::components::generated::reverse_uppercase::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?)
            .await
        }
        "scratch" => {
          crate::components::generated::scratch::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?)
            .await
        }
        "uppercase" => {
          crate::components::generated::uppercase::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?)
            .await
        }
        "validate" => {
          crate::components::generated::validate::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?)
            .await
        }
        "__batch__" => {
          crate::components::generated::__batch__::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?)
            .await
        }
        op => Err(format!("Component not found on this collection: {}", op).into()),
      }?;
      Ok(stream)
    })
  }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_signature() -> wasmflow_sdk::v1::types::CollectionSignature {
  let mut components: std::collections::HashMap<String, wasmflow_sdk::v1::types::ComponentSignature> =
    std::collections::HashMap::new();

  components.insert("error".to_owned(), generated::error::signature());
  components.insert("reverse".to_owned(), generated::reverse::signature());
  components.insert(
    "reverse-uppercase".to_owned(),
    generated::reverse_uppercase::signature(),
  );
  components.insert("scratch".to_owned(), generated::scratch::signature());
  components.insert("uppercase".to_owned(), generated::uppercase::signature());
  components.insert("validate".to_owned(), generated::validate::signature());

  wasmflow_sdk::v1::types::CollectionSignature {
    name: Some("test-component".to_owned()),
    features: wasmflow_sdk::v1::types::CollectionFeatures {
      streaming: false,
      stateful: false,
      version: wasmflow_sdk::v1::types::CollectionVersion::V0,
    },
    format: 1,
    version: "0.0.1".to_owned(),
    types: std::collections::HashMap::from([(
      "Unit".to_owned(),
      wasmflow_sdk::v1::types::TypeDefinition::Enum(wasmflow_sdk::v1::types::EnumSignature {
        name: "Unit".to_owned(),
        values: vec![
          wasmflow_sdk::v1::types::EnumVariant::new("0", 0),
          wasmflow_sdk::v1::types::EnumVariant::new("1", 1),
        ],
      }),
    )])
    .into(),
    components: components.into(),
    wellknown: Vec::new(),
    config: wasmflow_sdk::v1::types::TypeMap::new(),
  }
}

pub mod types {

  #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
  pub enum Unit {
    Millis,
    Micros,
  }
}
pub mod generated {

  // start component error
  pub mod error {
    // The user-facing implementation job impl.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};

    // The generated definition of inputs, outputs, config, et al.
    use super::error as definition;
    // The generated integration code between the definition and the implementation.
    use super::error as integration;
    use crate::components::error as implementation;

    #[derive(Default, Clone, Copy)]
    #[allow(missing_debug_implementations)]
    pub struct Component {}

    impl wasmflow_sdk::v1::Component for Component {
      type Inputs = definition::Inputs;
      type Outputs = definition::OutputPorts;
      type Config = integration::Config;
    }

    impl wasmflow_sdk::v1::ephemeral::BatchedJobExecutor for Component {
      #[cfg(not(target_arch = "wasm32"))]
      type Payload = wasmflow_sdk::v1::packet::v1::PacketMap;
      #[cfg(target_arch = "wasm32")]
      type Payload = wasmflow_sdk::v1::wasm::EncodedMap;
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::ephemeral::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
    pub fn signature() -> wasmflow_sdk::v1::types::ComponentSignature {
      wasmflow_sdk::v1::types::ComponentSignature {
        name: "error".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn convert_inputs(
      mut payload: wasmflow_sdk::v1::packet::v1::PacketMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        input: payload
          .remove("input")
          .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("input".to_owned()))?
          .deserialize()?,
      })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn convert_inputs(
      payload: wasmflow_sdk::v1::wasm::EncodedMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        input: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("input")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "input")]
      pub input: String,
    }

    impl From<Inputs> for wasmflow_sdk::v1::packet::PacketMap {
      fn from(inputs: Inputs) -> wasmflow_sdk::v1::packet::PacketMap {
        let mut map = std::collections::HashMap::default();
        map.insert(
          "input".to_owned(),
          wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.input).into(),
        );
        wasmflow_sdk::v1::packet::PacketMap::new(map)
      }
    }

    #[must_use]
    #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
    pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("input".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
      map
    }

    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(feature = "host")]
    pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("output".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg(feature = "host")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    impl OutputPorts {
      fn new(id: u32) -> Self {
        Self {
          output: OutputPortSender::new(id),
        }
      }
    }

    // Definition and implementation of each port's sender.
    #[derive(Debug)]
    #[cfg(feature = "host")]
    pub struct OutputPortSender {
      port: wasmflow_sdk::v1::PortChannel,
      id: u32,
    }

    #[cfg(feature = "host")]
    impl OutputPortSender {
      fn new(id: u32) -> Self {
        Self {
          id,
          port: wasmflow_sdk::v1::PortChannel::new("output"),
        }
      }
    }

    #[cfg(all(feature = "host"))]
    impl wasmflow_sdk::v1::Writable for OutputPortSender {
      type PayloadType = String;

      fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
        if self.port.is_closed() {
          Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
        } else {
          Ok(&self.port)
        }
      }

      fn get_port_name(&self) -> &str {
        &self.port.name
      }

      fn get_id(&self) -> u32 {
        self.id
      }
    }

    #[cfg(all(feature = "host"))]
    pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::v1::PacketStream) {
      let mut outputs = OutputPorts::new(id);
      let mut ports = vec![&mut outputs.output.port];
      let stream = wasmflow_sdk::v1::PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ComponentOutput,
    }

    impl Outputs {
      pub async fn output(&mut self) -> Result<wasmflow_sdk::v1::PortOutput<String>, wasmflow_sdk::v1::error::Error> {
        let packets = self.packets.drain_port("output").await?;
        Ok(wasmflow_sdk::v1::PortOutput::new("output".to_owned(), packets))
      }
    }

    impl From<ComponentOutput> for Outputs {
      fn from(packets: ComponentOutput) -> Self {
        Self { packets }
      }
    }

    impl From<wasmflow_sdk::v1::PacketStream> for Outputs {
      fn from(stream: wasmflow_sdk::v1::PacketStream) -> Self {
        Self {
          packets: ComponentOutput::new(stream),
        }
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl From<wasmflow_sdk::v1::transport::TransportStream> for Outputs {
      fn from(stream: wasmflow_sdk::v1::transport::TransportStream) -> Self {
        Self {
          packets: ComponentOutput::new_from_ts(stream),
        }
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component error
  // start component reverse
  pub mod reverse {
    // The user-facing implementation job impl.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};

    // The generated definition of inputs, outputs, config, et al.
    use super::reverse as definition;
    // The generated integration code between the definition and the implementation.
    use super::reverse as integration;
    use crate::components::reverse as implementation;

    #[derive(Default, Clone, Copy)]
    #[allow(missing_debug_implementations)]
    pub struct Component {}

    impl wasmflow_sdk::v1::Component for Component {
      type Inputs = definition::Inputs;
      type Outputs = definition::OutputPorts;
      type Config = integration::Config;
    }

    impl wasmflow_sdk::v1::ephemeral::BatchedJobExecutor for Component {
      #[cfg(not(target_arch = "wasm32"))]
      type Payload = wasmflow_sdk::v1::packet::v1::PacketMap;
      #[cfg(target_arch = "wasm32")]
      type Payload = wasmflow_sdk::v1::wasm::EncodedMap;
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::ephemeral::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
    pub fn signature() -> wasmflow_sdk::v1::types::ComponentSignature {
      wasmflow_sdk::v1::types::ComponentSignature {
        name: "reverse".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn convert_inputs(
      mut payload: wasmflow_sdk::v1::packet::v1::PacketMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        input: payload
          .remove("input")
          .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("input".to_owned()))?
          .deserialize()?,
      })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn convert_inputs(
      payload: wasmflow_sdk::v1::wasm::EncodedMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        input: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("input")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "input")]
      pub input: String,
    }

    impl From<Inputs> for wasmflow_sdk::v1::packet::PacketMap {
      fn from(inputs: Inputs) -> wasmflow_sdk::v1::packet::PacketMap {
        let mut map = std::collections::HashMap::default();
        map.insert(
          "input".to_owned(),
          wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.input).into(),
        );
        wasmflow_sdk::v1::packet::PacketMap::new(map)
      }
    }

    #[must_use]
    #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
    pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("input".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
      map
    }

    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(feature = "host")]
    pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("output".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg(feature = "host")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    impl OutputPorts {
      fn new(id: u32) -> Self {
        Self {
          output: OutputPortSender::new(id),
        }
      }
    }

    // Definition and implementation of each port's sender.
    #[derive(Debug)]
    #[cfg(feature = "host")]
    pub struct OutputPortSender {
      port: wasmflow_sdk::v1::PortChannel,
      id: u32,
    }

    #[cfg(feature = "host")]
    impl OutputPortSender {
      fn new(id: u32) -> Self {
        Self {
          id,
          port: wasmflow_sdk::v1::PortChannel::new("output"),
        }
      }
    }

    #[cfg(all(feature = "host"))]
    impl wasmflow_sdk::v1::Writable for OutputPortSender {
      type PayloadType = String;

      fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
        if self.port.is_closed() {
          Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
        } else {
          Ok(&self.port)
        }
      }

      fn get_port_name(&self) -> &str {
        &self.port.name
      }

      fn get_id(&self) -> u32 {
        self.id
      }
    }

    #[cfg(all(feature = "host"))]
    pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::v1::PacketStream) {
      let mut outputs = OutputPorts::new(id);
      let mut ports = vec![&mut outputs.output.port];
      let stream = wasmflow_sdk::v1::PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ComponentOutput,
    }

    impl Outputs {
      pub async fn output(&mut self) -> Result<wasmflow_sdk::v1::PortOutput<String>, wasmflow_sdk::v1::error::Error> {
        let packets = self.packets.drain_port("output").await?;
        Ok(wasmflow_sdk::v1::PortOutput::new("output".to_owned(), packets))
      }
    }

    impl From<ComponentOutput> for Outputs {
      fn from(packets: ComponentOutput) -> Self {
        Self { packets }
      }
    }

    impl From<wasmflow_sdk::v1::PacketStream> for Outputs {
      fn from(stream: wasmflow_sdk::v1::PacketStream) -> Self {
        Self {
          packets: ComponentOutput::new(stream),
        }
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl From<wasmflow_sdk::v1::transport::TransportStream> for Outputs {
      fn from(stream: wasmflow_sdk::v1::transport::TransportStream) -> Self {
        Self {
          packets: ComponentOutput::new_from_ts(stream),
        }
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component reverse
  // start component reverse-uppercase
  pub mod reverse_uppercase {
    // The user-facing implementation job impl.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};

    // The generated definition of inputs, outputs, config, et al.
    use super::reverse_uppercase as definition;
    // The generated integration code between the definition and the implementation.
    use super::reverse_uppercase as integration;
    use crate::components::reverse_uppercase as implementation;

    #[derive(Default, Clone, Copy)]
    #[allow(missing_debug_implementations)]
    pub struct Component {}

    impl wasmflow_sdk::v1::Component for Component {
      type Inputs = definition::Inputs;
      type Outputs = definition::OutputPorts;
      type Config = integration::Config;
    }

    impl wasmflow_sdk::v1::ephemeral::BatchedJobExecutor for Component {
      #[cfg(not(target_arch = "wasm32"))]
      type Payload = wasmflow_sdk::v1::packet::v1::PacketMap;
      #[cfg(target_arch = "wasm32")]
      type Payload = wasmflow_sdk::v1::wasm::EncodedMap;
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::ephemeral::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
    pub fn signature() -> wasmflow_sdk::v1::types::ComponentSignature {
      wasmflow_sdk::v1::types::ComponentSignature {
        name: "reverse-uppercase".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn convert_inputs(
      mut payload: wasmflow_sdk::v1::packet::v1::PacketMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        input: payload
          .remove("input")
          .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("input".to_owned()))?
          .deserialize()?,
        link: payload
          .remove("link")
          .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("link".to_owned()))?
          .deserialize()?,
      })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn convert_inputs(
      payload: wasmflow_sdk::v1::wasm::EncodedMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        input: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("input")?)?,
        link: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("link")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "input")]
      pub input: String,
      #[serde(rename = "link")]
      pub link: wasmflow_sdk::v1::CollectionLink,
    }

    impl From<Inputs> for wasmflow_sdk::v1::packet::PacketMap {
      fn from(inputs: Inputs) -> wasmflow_sdk::v1::packet::PacketMap {
        let mut map = std::collections::HashMap::default();
        map.insert(
          "input".to_owned(),
          wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.input).into(),
        );
        map.insert(
          "link".to_owned(),
          wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.link).into(),
        );
        wasmflow_sdk::v1::packet::PacketMap::new(map)
      }
    }

    #[must_use]
    #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
    pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("input".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
      map.insert(
        "link".to_owned(),
        wasmflow_sdk::v1::types::TypeSignature::Link { schemas: vec![] },
      );
      map
    }

    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(feature = "host")]
    pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("output".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg(feature = "host")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    impl OutputPorts {
      fn new(id: u32) -> Self {
        Self {
          output: OutputPortSender::new(id),
        }
      }
    }

    // Definition and implementation of each port's sender.
    #[derive(Debug)]
    #[cfg(feature = "host")]
    pub struct OutputPortSender {
      port: wasmflow_sdk::v1::PortChannel,
      id: u32,
    }

    #[cfg(feature = "host")]
    impl OutputPortSender {
      fn new(id: u32) -> Self {
        Self {
          id,
          port: wasmflow_sdk::v1::PortChannel::new("output"),
        }
      }
    }

    #[cfg(all(feature = "host"))]
    impl wasmflow_sdk::v1::Writable for OutputPortSender {
      type PayloadType = String;

      fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
        if self.port.is_closed() {
          Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
        } else {
          Ok(&self.port)
        }
      }

      fn get_port_name(&self) -> &str {
        &self.port.name
      }

      fn get_id(&self) -> u32 {
        self.id
      }
    }

    #[cfg(all(feature = "host"))]
    pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::v1::PacketStream) {
      let mut outputs = OutputPorts::new(id);
      let mut ports = vec![&mut outputs.output.port];
      let stream = wasmflow_sdk::v1::PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ComponentOutput,
    }

    impl Outputs {
      pub async fn output(&mut self) -> Result<wasmflow_sdk::v1::PortOutput<String>, wasmflow_sdk::v1::error::Error> {
        let packets = self.packets.drain_port("output").await?;
        Ok(wasmflow_sdk::v1::PortOutput::new("output".to_owned(), packets))
      }
    }

    impl From<ComponentOutput> for Outputs {
      fn from(packets: ComponentOutput) -> Self {
        Self { packets }
      }
    }

    impl From<wasmflow_sdk::v1::PacketStream> for Outputs {
      fn from(stream: wasmflow_sdk::v1::PacketStream) -> Self {
        Self {
          packets: ComponentOutput::new(stream),
        }
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl From<wasmflow_sdk::v1::transport::TransportStream> for Outputs {
      fn from(stream: wasmflow_sdk::v1::transport::TransportStream) -> Self {
        Self {
          packets: ComponentOutput::new_from_ts(stream),
        }
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component reverse-uppercase
  // start component scratch
  pub mod scratch {
    // The user-facing implementation job impl.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};

    // The generated definition of inputs, outputs, config, et al.
    use super::scratch as definition;
    // The generated integration code between the definition and the implementation.
    use super::scratch as integration;
    use crate::components::scratch as implementation;

    #[derive(Default, Clone, Copy)]
    #[allow(missing_debug_implementations)]
    pub struct Component {}

    impl wasmflow_sdk::v1::Component for Component {
      type Inputs = definition::Inputs;
      type Outputs = definition::OutputPorts;
      type Config = integration::Config;
    }

    impl wasmflow_sdk::v1::ephemeral::BatchedJobExecutor for Component {
      #[cfg(not(target_arch = "wasm32"))]
      type Payload = wasmflow_sdk::v1::packet::v1::PacketMap;
      #[cfg(target_arch = "wasm32")]
      type Payload = wasmflow_sdk::v1::wasm::EncodedMap;
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::ephemeral::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
    pub fn signature() -> wasmflow_sdk::v1::types::ComponentSignature {
      wasmflow_sdk::v1::types::ComponentSignature {
        name: "scratch".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn convert_inputs(
      mut payload: wasmflow_sdk::v1::packet::v1::PacketMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        name: payload
          .remove("name")
          .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("name".to_owned()))?
          .deserialize()?,
        age: payload
          .remove("age")
          .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("age".to_owned()))?
          .deserialize()?,
      })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn convert_inputs(
      payload: wasmflow_sdk::v1::wasm::EncodedMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        name: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("name")?)?,
        age: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("age")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "name")]
      pub name: String,
      #[serde(rename = "age")]
      pub age: i64,
    }

    impl From<Inputs> for wasmflow_sdk::v1::packet::PacketMap {
      fn from(inputs: Inputs) -> wasmflow_sdk::v1::packet::PacketMap {
        let mut map = std::collections::HashMap::default();
        map.insert(
          "name".to_owned(),
          wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.name).into(),
        );
        map.insert(
          "age".to_owned(),
          wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.age).into(),
        );
        wasmflow_sdk::v1::packet::PacketMap::new(map)
      }
    }

    #[must_use]
    #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
    pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("name".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
      map.insert("age".to_owned(), wasmflow_sdk::v1::types::TypeSignature::I64);
      map
    }

    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(feature = "host")]
    pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("message".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
      map.insert("age".to_owned(), wasmflow_sdk::v1::types::TypeSignature::I64);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg(feature = "host")]
    pub struct OutputPorts {
      pub message: MessagePortSender,
      pub age: AgePortSender,
    }

    impl OutputPorts {
      fn new(id: u32) -> Self {
        Self {
          message: MessagePortSender::new(id),
          age: AgePortSender::new(id),
        }
      }
    }

    // Definition and implementation of each port's sender.
    #[derive(Debug)]
    #[cfg(feature = "host")]
    pub struct MessagePortSender {
      port: wasmflow_sdk::v1::PortChannel,
      id: u32,
    }

    #[cfg(feature = "host")]
    impl MessagePortSender {
      fn new(id: u32) -> Self {
        Self {
          id,
          port: wasmflow_sdk::v1::PortChannel::new("message"),
        }
      }
    }

    #[cfg(all(feature = "host"))]
    impl wasmflow_sdk::v1::Writable for MessagePortSender {
      type PayloadType = String;

      fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
        if self.port.is_closed() {
          Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
        } else {
          Ok(&self.port)
        }
      }

      fn get_port_name(&self) -> &str {
        &self.port.name
      }

      fn get_id(&self) -> u32 {
        self.id
      }
    }

    #[derive(Debug)]
    #[cfg(feature = "host")]
    pub struct AgePortSender {
      port: wasmflow_sdk::v1::PortChannel,
      id: u32,
    }

    #[cfg(feature = "host")]
    impl AgePortSender {
      fn new(id: u32) -> Self {
        Self {
          id,
          port: wasmflow_sdk::v1::PortChannel::new("age"),
        }
      }
    }

    #[cfg(all(feature = "host"))]
    impl wasmflow_sdk::v1::Writable for AgePortSender {
      type PayloadType = i64;

      fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
        if self.port.is_closed() {
          Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
        } else {
          Ok(&self.port)
        }
      }

      fn get_port_name(&self) -> &str {
        &self.port.name
      }

      fn get_id(&self) -> u32 {
        self.id
      }
    }

    #[cfg(all(feature = "host"))]
    pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::v1::PacketStream) {
      let mut outputs = OutputPorts::new(id);
      let mut ports = vec![&mut outputs.message.port, &mut outputs.age.port];
      let stream = wasmflow_sdk::v1::PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ComponentOutput,
    }

    impl Outputs {
      pub async fn message(&mut self) -> Result<wasmflow_sdk::v1::PortOutput<String>, wasmflow_sdk::v1::error::Error> {
        let packets = self.packets.drain_port("message").await?;
        Ok(wasmflow_sdk::v1::PortOutput::new("message".to_owned(), packets))
      }
      pub async fn age(&mut self) -> Result<wasmflow_sdk::v1::PortOutput<i64>, wasmflow_sdk::v1::error::Error> {
        let packets = self.packets.drain_port("age").await?;
        Ok(wasmflow_sdk::v1::PortOutput::new("age".to_owned(), packets))
      }
    }

    impl From<ComponentOutput> for Outputs {
      fn from(packets: ComponentOutput) -> Self {
        Self { packets }
      }
    }

    impl From<wasmflow_sdk::v1::PacketStream> for Outputs {
      fn from(stream: wasmflow_sdk::v1::PacketStream) -> Self {
        Self {
          packets: ComponentOutput::new(stream),
        }
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl From<wasmflow_sdk::v1::transport::TransportStream> for Outputs {
      fn from(stream: wasmflow_sdk::v1::transport::TransportStream) -> Self {
        Self {
          packets: ComponentOutput::new_from_ts(stream),
        }
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {
      #[serde(rename = "unit")]
      pub unit: crate::components::types::Unit,
    }
  }
  // end component scratch
  // start component uppercase
  pub mod uppercase {
    // The user-facing implementation job impl.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};

    // The generated definition of inputs, outputs, config, et al.
    use super::uppercase as definition;
    // The generated integration code between the definition and the implementation.
    use super::uppercase as integration;
    use crate::components::uppercase as implementation;

    #[derive(Default, Clone, Copy)]
    #[allow(missing_debug_implementations)]
    pub struct Component {}

    impl wasmflow_sdk::v1::Component for Component {
      type Inputs = definition::Inputs;
      type Outputs = definition::OutputPorts;
      type Config = integration::Config;
    }

    impl wasmflow_sdk::v1::ephemeral::BatchedJobExecutor for Component {
      #[cfg(not(target_arch = "wasm32"))]
      type Payload = wasmflow_sdk::v1::packet::v1::PacketMap;
      #[cfg(target_arch = "wasm32")]
      type Payload = wasmflow_sdk::v1::wasm::EncodedMap;
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::ephemeral::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
    pub fn signature() -> wasmflow_sdk::v1::types::ComponentSignature {
      wasmflow_sdk::v1::types::ComponentSignature {
        name: "uppercase".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn convert_inputs(
      mut payload: wasmflow_sdk::v1::packet::v1::PacketMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        input: payload
          .remove("input")
          .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("input".to_owned()))?
          .deserialize()?,
      })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn convert_inputs(
      payload: wasmflow_sdk::v1::wasm::EncodedMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        input: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("input")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "input")]
      pub input: String,
    }

    impl From<Inputs> for wasmflow_sdk::v1::packet::PacketMap {
      fn from(inputs: Inputs) -> wasmflow_sdk::v1::packet::PacketMap {
        let mut map = std::collections::HashMap::default();
        map.insert(
          "input".to_owned(),
          wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.input).into(),
        );
        wasmflow_sdk::v1::packet::PacketMap::new(map)
      }
    }

    #[must_use]
    #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
    pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("input".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
      map
    }

    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(feature = "host")]
    pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("output".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg(feature = "host")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    impl OutputPorts {
      fn new(id: u32) -> Self {
        Self {
          output: OutputPortSender::new(id),
        }
      }
    }

    // Definition and implementation of each port's sender.
    #[derive(Debug)]
    #[cfg(feature = "host")]
    pub struct OutputPortSender {
      port: wasmflow_sdk::v1::PortChannel,
      id: u32,
    }

    #[cfg(feature = "host")]
    impl OutputPortSender {
      fn new(id: u32) -> Self {
        Self {
          id,
          port: wasmflow_sdk::v1::PortChannel::new("output"),
        }
      }
    }

    #[cfg(all(feature = "host"))]
    impl wasmflow_sdk::v1::Writable for OutputPortSender {
      type PayloadType = String;

      fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
        if self.port.is_closed() {
          Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
        } else {
          Ok(&self.port)
        }
      }

      fn get_port_name(&self) -> &str {
        &self.port.name
      }

      fn get_id(&self) -> u32 {
        self.id
      }
    }

    #[cfg(all(feature = "host"))]
    pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::v1::PacketStream) {
      let mut outputs = OutputPorts::new(id);
      let mut ports = vec![&mut outputs.output.port];
      let stream = wasmflow_sdk::v1::PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ComponentOutput,
    }

    impl Outputs {
      pub async fn output(&mut self) -> Result<wasmflow_sdk::v1::PortOutput<String>, wasmflow_sdk::v1::error::Error> {
        let packets = self.packets.drain_port("output").await?;
        Ok(wasmflow_sdk::v1::PortOutput::new("output".to_owned(), packets))
      }
    }

    impl From<ComponentOutput> for Outputs {
      fn from(packets: ComponentOutput) -> Self {
        Self { packets }
      }
    }

    impl From<wasmflow_sdk::v1::PacketStream> for Outputs {
      fn from(stream: wasmflow_sdk::v1::PacketStream) -> Self {
        Self {
          packets: ComponentOutput::new(stream),
        }
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl From<wasmflow_sdk::v1::transport::TransportStream> for Outputs {
      fn from(stream: wasmflow_sdk::v1::transport::TransportStream) -> Self {
        Self {
          packets: ComponentOutput::new_from_ts(stream),
        }
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component uppercase
  // start component validate
  pub mod validate {
    // The user-facing implementation job impl.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};

    // The generated definition of inputs, outputs, config, et al.
    use super::validate as definition;
    // The generated integration code between the definition and the implementation.
    use super::validate as integration;
    use crate::components::validate as implementation;

    #[derive(Default, Clone, Copy)]
    #[allow(missing_debug_implementations)]
    pub struct Component {}

    impl wasmflow_sdk::v1::Component for Component {
      type Inputs = definition::Inputs;
      type Outputs = definition::OutputPorts;
      type Config = integration::Config;
    }

    impl wasmflow_sdk::v1::ephemeral::BatchedJobExecutor for Component {
      #[cfg(not(target_arch = "wasm32"))]
      type Payload = wasmflow_sdk::v1::packet::v1::PacketMap;
      #[cfg(target_arch = "wasm32")]
      type Payload = wasmflow_sdk::v1::wasm::EncodedMap;
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::ephemeral::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
    pub fn signature() -> wasmflow_sdk::v1::types::ComponentSignature {
      wasmflow_sdk::v1::types::ComponentSignature {
        name: "validate".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn convert_inputs(
      mut payload: wasmflow_sdk::v1::packet::v1::PacketMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        input: payload
          .remove("input")
          .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("input".to_owned()))?
          .deserialize()?,
      })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn convert_inputs(
      payload: wasmflow_sdk::v1::wasm::EncodedMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        input: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("input")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "input")]
      pub input: String,
    }

    impl From<Inputs> for wasmflow_sdk::v1::packet::PacketMap {
      fn from(inputs: Inputs) -> wasmflow_sdk::v1::packet::PacketMap {
        let mut map = std::collections::HashMap::default();
        map.insert(
          "input".to_owned(),
          wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.input).into(),
        );
        wasmflow_sdk::v1::packet::PacketMap::new(map)
      }
    }

    #[must_use]
    #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
    pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("input".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
      map
    }

    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(feature = "host")]
    pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("output".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg(feature = "host")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    impl OutputPorts {
      fn new(id: u32) -> Self {
        Self {
          output: OutputPortSender::new(id),
        }
      }
    }

    // Definition and implementation of each port's sender.
    #[derive(Debug)]
    #[cfg(feature = "host")]
    pub struct OutputPortSender {
      port: wasmflow_sdk::v1::PortChannel,
      id: u32,
    }

    #[cfg(feature = "host")]
    impl OutputPortSender {
      fn new(id: u32) -> Self {
        Self {
          id,
          port: wasmflow_sdk::v1::PortChannel::new("output"),
        }
      }
    }

    #[cfg(all(feature = "host"))]
    impl wasmflow_sdk::v1::Writable for OutputPortSender {
      type PayloadType = String;

      fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
        if self.port.is_closed() {
          Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
        } else {
          Ok(&self.port)
        }
      }

      fn get_port_name(&self) -> &str {
        &self.port.name
      }

      fn get_id(&self) -> u32 {
        self.id
      }
    }

    #[cfg(all(feature = "host"))]
    pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::v1::PacketStream) {
      let mut outputs = OutputPorts::new(id);
      let mut ports = vec![&mut outputs.output.port];
      let stream = wasmflow_sdk::v1::PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ComponentOutput,
    }

    impl Outputs {
      pub async fn output(&mut self) -> Result<wasmflow_sdk::v1::PortOutput<String>, wasmflow_sdk::v1::error::Error> {
        let packets = self.packets.drain_port("output").await?;
        Ok(wasmflow_sdk::v1::PortOutput::new("output".to_owned(), packets))
      }
    }

    impl From<ComponentOutput> for Outputs {
      fn from(packets: ComponentOutput) -> Self {
        Self { packets }
      }
    }

    impl From<wasmflow_sdk::v1::PacketStream> for Outputs {
      fn from(stream: wasmflow_sdk::v1::PacketStream) -> Self {
        Self {
          packets: ComponentOutput::new(stream),
        }
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl From<wasmflow_sdk::v1::transport::TransportStream> for Outputs {
      fn from(stream: wasmflow_sdk::v1::transport::TransportStream) -> Self {
        Self {
          packets: ComponentOutput::new_from_ts(stream),
        }
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component validate

  pub mod __batch__ {
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};

    use super::{__batch__ as integration, __batch__ as definition};
    use crate::components::__batch__ as implementation;

    impl wasmflow_sdk::v1::ephemeral::BatchedJobExecutor for Component {
      #[cfg(not(target_arch = "wasm32"))]
      type Payload = wasmflow_sdk::v1::packet::v1::PacketMap;
      #[cfg(target_arch = "wasm32")]
      type Payload = wasmflow_sdk::v1::wasm::EncodedMap;
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::ephemeral::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub enum ComponentInputs {
      Error(super::error::Inputs),
      Reverse(super::reverse::Inputs),
      ReverseUppercase(super::reverse_uppercase::Inputs),
      Scratch(super::scratch::Inputs),
      Uppercase(super::uppercase::Inputs),
      Validate(super::validate::Inputs),
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub enum ComponentOutputs {
      Error(super::error::Outputs),
      Reverse(super::reverse::Outputs),
      ReverseUppercase(super::reverse_uppercase::Outputs),
      Scratch(super::scratch::Outputs),
      Uppercase(super::uppercase::Outputs),
      Validate(super::validate::Outputs),
    }

    #[derive(Debug, serde::Deserialize)]
    pub enum Config {
      Error(super::error::Config),
      Reverse(super::reverse::Config),
      ReverseUppercase(super::reverse_uppercase::Config),
      Scratch(super::scratch::Config),
      Uppercase(super::uppercase::Config),
      Validate(super::validate::Config),
    }

    #[derive(Default, Clone, Copy)]
    #[allow(missing_debug_implementations)]
    pub struct Component {}

    impl wasmflow_sdk::v1::Component for Component {
      type Inputs = definition::Inputs;
      type Outputs = definition::OutputPorts;
      type Config = integration::Config;
    }
    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(feature = "host")]
    pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("result".to_owned(), wasmflow_sdk::v1::types::TypeSignature::Bool);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg(feature = "host")]
    pub struct OutputPorts {
      pub result: ResultPortSender,
    }

    impl OutputPorts {
      fn new(id: u32) -> Self {
        Self {
          result: ResultPortSender::new(id),
        }
      }
    }

    // Definition and implementation of each port's sender.
    #[derive(Debug)]
    #[cfg(feature = "host")]
    pub struct ResultPortSender {
      port: wasmflow_sdk::v1::PortChannel,
      id: u32,
    }

    #[cfg(feature = "host")]
    impl ResultPortSender {
      fn new(id: u32) -> Self {
        Self {
          id,
          port: wasmflow_sdk::v1::PortChannel::new("result"),
        }
      }
    }

    #[cfg(all(feature = "host"))]
    impl wasmflow_sdk::v1::Writable for ResultPortSender {
      type PayloadType = bool;

      fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
        if self.port.is_closed() {
          Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
        } else {
          Ok(&self.port)
        }
      }

      fn get_port_name(&self) -> &str {
        &self.port.name
      }

      fn get_id(&self) -> u32 {
        self.id
      }
    }

    #[cfg(all(feature = "host"))]
    pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::v1::PacketStream) {
      let mut outputs = OutputPorts::new(id);
      let mut ports = vec![&mut outputs.result.port];
      let stream = wasmflow_sdk::v1::PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ComponentOutput,
    }

    impl Outputs {
      pub async fn result(&mut self) -> Result<wasmflow_sdk::v1::PortOutput<bool>, wasmflow_sdk::v1::error::Error> {
        let packets = self.packets.drain_port("result").await?;
        Ok(wasmflow_sdk::v1::PortOutput::new("result".to_owned(), packets))
      }
    }

    impl From<ComponentOutput> for Outputs {
      fn from(packets: ComponentOutput) -> Self {
        Self { packets }
      }
    }

    impl From<wasmflow_sdk::v1::PacketStream> for Outputs {
      fn from(stream: wasmflow_sdk::v1::PacketStream) -> Self {
        Self {
          packets: ComponentOutput::new(stream),
        }
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl From<wasmflow_sdk::v1::transport::TransportStream> for Outputs {
      fn from(stream: wasmflow_sdk::v1::transport::TransportStream) -> Self {
        Self {
          packets: ComponentOutput::new_from_ts(stream),
        }
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn convert_inputs(
      mut payload: wasmflow_sdk::v1::packet::v1::PacketMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        inputs: payload
          .remove("inputs")
          .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("inputs".to_owned()))?
          .deserialize()?,
      })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn convert_inputs(
      payload: wasmflow_sdk::v1::wasm::EncodedMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        inputs: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("inputs")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "inputs")]
      pub inputs: Vec<ComponentInputs>,
    }

    impl From<Inputs> for wasmflow_sdk::v1::packet::PacketMap {
      fn from(inputs: Inputs) -> wasmflow_sdk::v1::packet::PacketMap {
        let mut map = std::collections::HashMap::default();
        map.insert(
          "inputs".to_owned(),
          wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.inputs).into(),
        );
        wasmflow_sdk::v1::packet::PacketMap::new(map)
      }
    }

    #[must_use]
    #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
    pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert(
        "inputs".to_owned(),
        wasmflow_sdk::v1::types::TypeSignature::List {
          element: Box::new(wasmflow_sdk::v1::types::TypeSignature::Internal(
            wasmflow_sdk::v1::types::InternalType::ComponentInput,
          )),
        },
      );
      map
    }
  }
}
