/**********************************************
***** This file is generated, do not edit *****
***********************************************/
#![allow(
  unused_qualifications,
  unused_imports,
  missing_copy_implementations,
  unused_qualifications
)]

use wasmflow_sdk::sdk::ephemeral::BatchedJobExecutor;

#[cfg(all(target_arch = "wasm32"))]
type CallResult = wasmflow_sdk::sdk::BoxedFuture<Result<Vec<u8>, wasmflow_sdk::sdk::BoxedError>>;

#[cfg(all(target_arch = "wasm32"))]
#[allow(unsafe_code)]
#[no_mangle]
pub(crate) extern "C" fn wapc_init() {
  wasmflow_sdk::sdk::wasm::runtime::register_dispatcher(Box::new(ComponentDispatcher::default()));
}

pub mod __batch__;
pub mod fs_read; // fs-read

#[allow(unused)]
static ALL_COMPONENTS: &[&str] = &["fs-read"];

#[derive(Default, Copy, Clone)]
#[allow(missing_debug_implementations)]
pub struct ComponentDispatcher {}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::too_many_lines)]
impl wasmflow_sdk::sdk::ephemeral::WasmDispatcher for ComponentDispatcher {
  fn dispatch(&self, op: &'static str, payload: &'static [u8]) -> CallResult {
    Box::pin(async move {
      let (mut stream, id) = match op {
        "fs-read" => {
          crate::components::generated::fs_read::Component::default()
            .execute(wasmflow_sdk::sdk::payload::from_buffer(payload)?)
            .await
        }
        _ => Err(wasmflow_sdk::error::Error::ComponentNotFound(op.to_owned(), ALL_COMPONENTS.join(", ")).into()),
      }?;
      while let Some(next) = wasmflow_sdk::sdk::StreamExt::next(&mut stream).await {
        wasmflow_sdk::sdk::wasm::port_send(&next.port, id, next.payload)?;
      }

      Ok(Vec::new())
    })
  }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_lines)]
impl wasmflow_sdk::sdk::ephemeral::NativeDispatcher for ComponentDispatcher {
  fn dispatch(
    &self,
    invocation: wasmflow_sdk::sdk::Invocation,
  ) -> wasmflow_sdk::sdk::BoxedFuture<Result<wasmflow_sdk::types::PacketStream, wasmflow_sdk::sdk::BoxedError>> {
    Box::pin(async move {
      let (stream, _id) = match invocation.target.name() {
        "fs-read" => {
          crate::components::generated::fs_read::Component::default()
            .execute(wasmflow_sdk::sdk::payload::from_invocation(invocation)?)
            .await
        }
        "__batch__" => {
          crate::components::generated::__batch__::Component::default()
            .execute(wasmflow_sdk::sdk::payload::from_invocation(invocation)?)
            .await
        }
        op => Err(format!("Component not found on this provider: {}", op).into()),
      }?;
      Ok(stream)
    })
  }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_signature() -> wasmflow_sdk::types::ProviderSignature {
  let mut components: std::collections::HashMap<String, wasmflow_sdk::types::ComponentSignature> =
    std::collections::HashMap::new();

  components.insert("fs-read".to_owned(), generated::fs_read::signature());

  wasmflow_sdk::types::ProviderSignature {
    name: Some("test-wasi-component".to_owned()),
    format: 1,
    version: "0.0.1".to_owned(),
    types: std::collections::HashMap::from([]).into(),
    components: components.into(),
    wellknown: Vec::new(),
    config: wasmflow_sdk::types::TypeMap::new(),
  }
}

pub mod types {
  // no additional types
}
pub mod generated {

  // start component fs-read
  pub mod fs_read {
    // The user-facing implementation for State and job impl.
    pub use wasmflow_sdk::console_log;
    pub use wasmflow_sdk::packet::v1::Packet;
    pub use wasmflow_sdk::sdk::{ProviderOutput, Writable};

    // The generated definition of inputs, outputs, config, et al.
    use super::fs_read as definition;
    // The generated integration code between the definition and the implementation.
    use super::fs_read as integration;
    use crate::components::fs_read as implementation;

    #[derive(Default, Clone, Copy)]
    #[allow(missing_debug_implementations)]
    pub struct Component {}

    impl wasmflow_sdk::sdk::Component for Component {
      type Inputs = definition::Inputs;
      type Outputs = definition::OutputPorts;
      type Config = integration::Config;
    }

    impl wasmflow_sdk::sdk::ephemeral::BatchedJobExecutor for Component {
      #[cfg(not(target_arch = "wasm32"))]
      type Payload = wasmflow_sdk::packet::v1::PacketMap;
      #[cfg(target_arch = "wasm32")]
      type Payload = wasmflow_sdk::sdk::wasm::EncodedMap;
      type State = implementation::State;
      type Config = Config;
      type Return = (wasmflow_sdk::types::PacketStream, u32);

      fn execute(
        &self,
        payload: wasmflow_sdk::sdk::IncomingPayload<Self::Payload, Self::Config, Self::State>,
      ) -> wasmflow_sdk::sdk::BoxedFuture<Result<Self::Return, wasmflow_sdk::sdk::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::sdk::ephemeral::BatchedComponent;
          let id = payload.id();
          let (outputs, mut stream) = definition::get_outputs(id);
          let (payload, config, state) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          let new_state = Component::job(inputs, outputs, state, config).await?;
          stream.push(wasmflow_sdk::packet::PacketWrapper::state(
            Packet::success(&new_state).into(),
          ));
          Ok((stream, id))
        })
      }
    }

    #[cfg(all(feature = "provider", not(target_arch = "wasm32")))]
    pub fn signature() -> wasmflow_sdk::types::ComponentSignature {
      wasmflow_sdk::types::ComponentSignature {
        name: "fs-read".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn convert_inputs(
      mut payload: wasmflow_sdk::packet::v1::PacketMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        filename: payload
          .remove("filename")
          .ok_or_else(|| wasmflow_sdk::error::Error::MissingInput("filename".to_owned()))?
          .deserialize()?,
      })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn convert_inputs(
      payload: wasmflow_sdk::sdk::wasm::EncodedMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        filename: wasmflow_sdk::codec::messagepack::deserialize(payload.get("filename")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "filename")]
      pub filename: String,
    }

    impl From<Inputs> for wasmflow_sdk::packet::PacketMap {
      fn from(inputs: Inputs) -> wasmflow_sdk::packet::PacketMap {
        let mut map = std::collections::HashMap::default();
        map.insert(
          "filename".to_owned(),
          wasmflow_sdk::packet::v1::Packet::success(&inputs.filename).into(),
        );
        wasmflow_sdk::packet::PacketMap::new(map)
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", not(target_arch = "wasm32")))]
    pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("filename".to_owned(), wasmflow_sdk::types::TypeSignature::String);
      map
    }

    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(all(feature = "provider"))]
    pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("contents".to_owned(), wasmflow_sdk::types::TypeSignature::String);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct OutputPorts {
      pub contents: ContentsPortSender,
    }

    impl OutputPorts {
      fn new(id: u32) -> Self {
        Self {
          contents: ContentsPortSender::new(id),
        }
      }
    }

    // Definition and implementation of each port's sender.
    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct ContentsPortSender {
      port: wasmflow_sdk::sdk::PortChannel,
      id: u32,
    }

    #[cfg(all(feature = "provider"))]
    impl ContentsPortSender {
      fn new(id: u32) -> Self {
        Self {
          id,
          port: wasmflow_sdk::sdk::PortChannel::new("contents"),
        }
      }
    }

    #[cfg(all(feature = "provider"))]
    impl wasmflow_sdk::sdk::Writable for ContentsPortSender {
      type PayloadType = String;

      fn get_port(&self) -> Result<&wasmflow_sdk::sdk::PortChannel, wasmflow_sdk::sdk::BoxedError> {
        if self.port.is_closed() {
          Err(Box::new(wasmflow_sdk::error::Error::SendError("@key".to_owned())))
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

    #[cfg(all(feature = "provider"))]
    pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::types::PacketStream) {
      let mut outputs = OutputPorts::new(id);
      let mut ports = vec![&mut outputs.contents.port];
      let stream = wasmflow_sdk::sdk::PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    impl Outputs {
      pub async fn contents(&mut self) -> Result<wasmflow_sdk::sdk::PortOutput<String>, wasmflow_sdk::error::Error> {
        let packets = self.packets.drain_port("contents").await?;
        Ok(wasmflow_sdk::sdk::PortOutput::new("contents".to_owned(), packets))
      }
    }

    impl From<ProviderOutput> for Outputs {
      fn from(packets: ProviderOutput) -> Self {
        Self { packets }
      }
    }

    impl From<wasmflow_sdk::types::PacketStream> for Outputs {
      fn from(stream: wasmflow_sdk::types::PacketStream) -> Self {
        Self {
          packets: ProviderOutput::new(stream),
        }
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl From<wasmflow_sdk::types::TransportStream> for Outputs {
      fn from(stream: wasmflow_sdk::types::TransportStream) -> Self {
        Self {
          packets: ProviderOutput::new_from_ts(stream),
        }
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component fs-read

  pub mod __batch__ {
    pub use wasmflow_sdk::console_log;
    pub use wasmflow_sdk::packet::v1::Packet;
    pub use wasmflow_sdk::sdk::{ProviderOutput, Writable};

    use super::{__batch__ as integration, __batch__ as definition};
    use crate::components::__batch__ as implementation;

    impl wasmflow_sdk::sdk::ephemeral::BatchedJobExecutor for Component {
      #[cfg(not(target_arch = "wasm32"))]
      type Payload = wasmflow_sdk::packet::v1::PacketMap;
      #[cfg(target_arch = "wasm32")]
      type Payload = wasmflow_sdk::sdk::wasm::EncodedMap;
      type State = implementation::State;
      type Config = Config;
      type Return = (wasmflow_sdk::types::PacketStream, u32);

      fn execute(
        &self,
        payload: wasmflow_sdk::sdk::IncomingPayload<Self::Payload, Self::Config, Self::State>,
      ) -> wasmflow_sdk::sdk::BoxedFuture<Result<Self::Return, wasmflow_sdk::sdk::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::sdk::ephemeral::BatchedComponent;
          let id = payload.id();
          let (outputs, mut stream) = definition::get_outputs(id);
          let (payload, config, state) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          let new_state = Component::job(inputs, outputs, state, config).await?;
          stream.push(wasmflow_sdk::packet::PacketWrapper::state(
            Packet::success(&new_state).into(),
          ));
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub enum ComponentInputs {
      FsRead(super::fs_read::Inputs),
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub enum ComponentOutputs {
      FsRead(super::fs_read::Outputs),
    }

    #[derive(Debug, serde::Deserialize)]
    pub enum Config {
      FsRead(super::fs_read::Config),
    }

    #[derive(Default, Clone, Copy)]
    #[allow(missing_debug_implementations)]
    pub struct Component {}

    impl wasmflow_sdk::sdk::Component for Component {
      type Inputs = definition::Inputs;
      type Outputs = definition::OutputPorts;
      type Config = integration::Config;
    }
    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(all(feature = "provider"))]
    pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("result".to_owned(), wasmflow_sdk::types::TypeSignature::Bool);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg(feature = "provider")]
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
    #[cfg(feature = "provider")]
    pub struct ResultPortSender {
      port: wasmflow_sdk::sdk::PortChannel,
      id: u32,
    }

    #[cfg(all(feature = "provider"))]
    impl ResultPortSender {
      fn new(id: u32) -> Self {
        Self {
          id,
          port: wasmflow_sdk::sdk::PortChannel::new("result"),
        }
      }
    }

    #[cfg(all(feature = "provider"))]
    impl wasmflow_sdk::sdk::Writable for ResultPortSender {
      type PayloadType = bool;

      fn get_port(&self) -> Result<&wasmflow_sdk::sdk::PortChannel, wasmflow_sdk::sdk::BoxedError> {
        if self.port.is_closed() {
          Err(Box::new(wasmflow_sdk::error::Error::SendError("@key".to_owned())))
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

    #[cfg(all(feature = "provider"))]
    pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::types::PacketStream) {
      let mut outputs = OutputPorts::new(id);
      let mut ports = vec![&mut outputs.result.port];
      let stream = wasmflow_sdk::sdk::PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    impl Outputs {
      pub async fn result(&mut self) -> Result<wasmflow_sdk::sdk::PortOutput<bool>, wasmflow_sdk::error::Error> {
        let packets = self.packets.drain_port("result").await?;
        Ok(wasmflow_sdk::sdk::PortOutput::new("result".to_owned(), packets))
      }
    }

    impl From<ProviderOutput> for Outputs {
      fn from(packets: ProviderOutput) -> Self {
        Self { packets }
      }
    }

    impl From<wasmflow_sdk::types::PacketStream> for Outputs {
      fn from(stream: wasmflow_sdk::types::PacketStream) -> Self {
        Self {
          packets: ProviderOutput::new(stream),
        }
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl From<wasmflow_sdk::types::TransportStream> for Outputs {
      fn from(stream: wasmflow_sdk::types::TransportStream) -> Self {
        Self {
          packets: ProviderOutput::new_from_ts(stream),
        }
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn convert_inputs(
      mut payload: wasmflow_sdk::packet::v1::PacketMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        inputs: payload
          .remove("inputs")
          .ok_or_else(|| wasmflow_sdk::error::Error::MissingInput("inputs".to_owned()))?
          .deserialize()?,
      })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn convert_inputs(
      payload: wasmflow_sdk::sdk::wasm::EncodedMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        inputs: wasmflow_sdk::codec::messagepack::deserialize(payload.get("inputs")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "inputs")]
      pub inputs: Vec<ComponentInputs>,
    }

    impl From<Inputs> for wasmflow_sdk::packet::PacketMap {
      fn from(inputs: Inputs) -> wasmflow_sdk::packet::PacketMap {
        let mut map = std::collections::HashMap::default();
        map.insert(
          "inputs".to_owned(),
          wasmflow_sdk::packet::v1::Packet::success(&inputs.inputs).into(),
        );
        wasmflow_sdk::packet::PacketMap::new(map)
      }
    }

    #[must_use]
    #[cfg(all(feature = "provider", not(target_arch = "wasm32")))]
    pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert(
        "inputs".to_owned(),
        wasmflow_sdk::types::TypeSignature::List {
          element: Box::new(wasmflow_sdk::types::TypeSignature::Internal(
            wasmflow_sdk::types::InternalType::ComponentInput,
          )),
        },
      );
      map
    }
  }
}
