#[macro_export]
macro_rules! native_actor {(
  $component_name:ident,
  fn job($inputs_name:ident:Inputs, $outputs_name:ident:Outputs) -> Result<Signal> $fun:block
) => {

    use crate::Result;
    use crate::components::native_component_actor::{NativeActor,NativeCallback};
    use crate::components::vino_component::NativeComponent;
    use vino_guest::Signal;

    use super::generated::$component_name::{Inputs, Outputs, InputEncoded, get_outputs, inputs_list, outputs_list, deserialize_inputs};

    pub struct Actor {
      callback: Option<NativeCallback>
    }

    impl<'a> Default for Actor {
      fn default() -> Self {
        Self {
          callback: None,
        }
      }
    }

    impl Actor {
      pub fn new(cb: NativeCallback) -> Self {
        Actor {
          callback: Some(cb)
        }
      }
    }

    impl NativeActor for Actor {
      fn get_def(&self) -> NativeComponent {
        NativeComponent {
          id: self.get_name(),
          inputs: self.get_input_ports(),
          outputs: self.get_output_ports()
        }
      }
      fn get_name(&self) -> String {
        format!("vino::{}",std::stringify!($component_name))
      }
      fn get_input_ports(&self) -> Vec<String> {
        inputs_list()
      }
      fn get_output_ports(&self) -> Vec<String> {
        outputs_list()
      }

      fn job_wrapper(&self, data: &[u8]) -> Result<Signal>{
        match &self.callback {
          None => Err(anyhow!("No callback registered with native actor").into()),
          Some(callback) => {
            let (inv_id, input_encoded) : (String, InputEncoded) = crate::deserialize(&data)?;
            let inputs = deserialize_inputs(input_encoded)?;
            let outputs = get_outputs(callback, inv_id);
            match job(inputs, outputs) {
              Ok(data) => Ok(data),
              Err(e) => Err(anyhow!("Error executing job: {}", e).into())
            }
          },
        }
      }
    }

    pub(crate) fn job($inputs_name: Inputs, $outputs_name: Outputs) -> Result<Signal> $fun
}}
