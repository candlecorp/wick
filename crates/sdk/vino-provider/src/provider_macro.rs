#[macro_export]
macro_rules! provider_component {(
  $component_name:ident,
  fn job($inputs_name:ident:Inputs, $outputs_name:ident:Outputs, $context_name: ident: Context<$context_type:ty>) -> $result_type:ty $fun:block
) => {
    use std::collections::HashMap;
    use log::{debug,trace};
    use vino_provider::{error::ProviderError, Context as ProviderContext};
    use vino_provider::VinoProviderComponent;
    use vino_rpc::port::{Sender, Receiver};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    use super::generated::$component_name::{Outputs, InputEncoded, get_outputs, inputs_list, outputs_list, deserialize_inputs};
    pub(crate) use super::generated::$component_name::{Inputs};

    pub(crate) struct Component {}

    impl Default for Component {
      fn default() -> Self {
        Self {
        }
      }
    }

    #[async_trait]
    impl VinoProviderComponent for Component {
      type Context = $context_type;

      fn get_name(&self) -> String {
        format!("vino::{}",std::stringify!($component_name))
      }
      fn get_input_ports(&self) -> Vec<String> {
        inputs_list()
      }
      fn get_output_ports(&self) -> Vec<String> {
        outputs_list()
      }
      async fn job_wrapper(&self, context:ProviderContext<Self::Context>, data: HashMap<String,Vec<u8>>) -> std::result::Result<Receiver, Box<dyn std::error::Error + Send + Sync>> {
          trace!("Job passed data: {:?}", data);
          let inputs = deserialize_inputs(&data).map_err(ProviderError::InputDeserializationError)?;
          let (outputs, receiver) = get_outputs();
          let result : $result_type = job(inputs, outputs, context).await;
          match result {
            Ok(_) => Ok(receiver),
            Err(e) => Err(ProviderError::JobError("Job failed".to_string()).into())
          }
      }

    }

    pub(crate) async fn job($inputs_name: Inputs, $outputs_name: Outputs, $context_name: ProviderContext<$context_type>) -> $result_type $fun
}}
