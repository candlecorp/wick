/**********************************************
***** This file is generated, do not edit *****
***********************************************/

use vino_provider::native::prelude::*;

use crate::generated;

pub(crate) fn get_component(
  name: &str,
) -> Option<Box<dyn NativeComponent<State = crate::State> + Sync + Send>> {
  match name {
    "test-component" => Some(Box::new(generated::test_component::Component::default())),
    _ => None,
  }
}

pub(crate) fn get_all_components() -> Vec<ComponentSignature> {
  vec![ComponentSignature {
    name: "test-component".to_owned(),
    inputs: generated::test_component::inputs_list()
      .into_iter()
      .map(From::from)
      .collect(),
    outputs: generated::test_component::outputs_list()
      .into_iter()
      .map(From::from)
      .collect(),
  }]
}

pub(crate) mod test_component {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  pub(crate) use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type State = crate::State;
    async fn execute(
      &self,
      context: Context<Self::State>,
      data: TransportMap,
    ) -> Result<MessageTransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| {
        NativeComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::test_component::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }

  pub(crate) fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      input: payload.consume("input")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    #[serde(rename = "input")]
    pub(crate) input: String,
  }

  #[must_use]
  pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("input", "string")]
  }

  #[derive(Debug, Default)]
  pub(crate) struct Outputs {
    pub(crate) output: OutputPortSender,
  }

  #[must_use]
  pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("output", "string")]
  }

  #[derive(Debug)]
  pub(crate) struct OutputPortSender {
    port: PortChannel,
  }
  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("output".into()),
      }
    }
  }
  impl PortSender for OutputPortSender {
    type PayloadType = String;

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
  pub(crate) fn get_outputs() -> (Outputs, MessageTransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }
}
