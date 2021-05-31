#[macro_export]
macro_rules! native_actor {(
  $component_name:ident,
  fn job($inputs_name:ident:Inputs, $outputs_name:ident:Outputs) -> Result<Signal> $fun:block
) => {

    use crate::Result;
    use crate::connection_downstream::ConnectionDownstream;
    use crate::native_component_actor::NativeActor;
    use crate::vino_component::NativeComponent;
    use crate::network::ActorPorts;
    use serde::Deserialize;
    use vino_guest::Signal;

    use super::generated::$component_name::{Inputs, Outputs, InputEncoded, get_outputs, inputs_list, outputs_list, deserialize_inputs};


    #[derive(Deserialize)]
    pub struct JobEncoded {
      #[serde(rename = "connection")]
      pub connection: ConnectionDownstream,
      #[serde(rename = "input")]
      pub input: InputEncoded,
    }

    pub struct Actor {}

    impl NativeActor for Actor {
      fn get_def(&self) -> NativeComponent {
        NativeComponent {
          id: self.get_name(),
          ports: ActorPorts{
            inputs: self.get_input_ports(),
            outputs: self.get_output_ports()
          },
          addr:None,
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
        let msg : JobEncoded = crate::deserialize(&data)?;
        let inputs = deserialize_inputs(msg.input)?;
        let outputs = get_outputs(msg.connection);
        match job(inputs, outputs) {
          Ok(data) => Ok(data),
          Err(e) => Err(anyhow!("Error executing job: {}", e).into())
        }
      }
    }

    pub(crate) fn job($inputs_name: Inputs, $outputs_name: Outputs) -> Result<Signal> $fun
}}
