#[macro_export]
macro_rules! provider_component {(
  $component_name:ident,
  fn job($inputs_name:ident:Inputs, $outputs_name:ident:Outputs, $context_name: ident: Context<$context_type:ty>) -> Result<()> $fun:block
) => {
    use log::{debug,trace};
    use vino_provider::{Result, error::ProviderError, Context as ProviderContext};
    use vino_provider::VinoProviderComponent;
    use vino_provider::port::{OutputStreams, Sender, Receiver};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    use super::generated::$component_name::{Outputs, InputEncoded, get_outputs, inputs_list, outputs_list, deserialize_inputs};
    pub use super::generated::$component_name::{Inputs};

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
      async fn job_wrapper(&self, context:ProviderContext<Self::Context>, data: &[u8]) -> Result<Receiver> {
          trace!("Job passed data: {:?}", data);
          let (inv_id, input_encoded) : (String, InputEncoded) = vino_runtime::deserialize(&data)?;
          debug!("Invocation ID: {:?}", inv_id);
          let inputs = deserialize_inputs(input_encoded).map_err(ProviderError::InputDeserializationError)?;
          let (outputs, receiver) = get_outputs();
          job(inputs, outputs, context).await?;
          Ok(receiver)
      }

    }

    pub(crate) async fn job($inputs_name: Inputs, $outputs_name: Outputs, $context_name: ProviderContext<$context_type>) -> Result<()> $fun
}}
