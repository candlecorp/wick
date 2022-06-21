use wasmflow_sdk::v1::packet::PacketMap;

pub use crate::components::generated::main::*;
use crate::components::*;

pub(crate) type State = ();

#[async_trait::async_trait]
impl wasmflow_sdk::v1::ephemeral::BatchedComponent for Component {
  type State = State;
  async fn job(
    mut input: Self::Inputs,
    output: Self::Outputs,
    state: Option<Self::State>,
    _config: Option<Self::Config>,
  ) -> Result<Option<Self::State>, Box<dyn std::error::Error + Send + Sync>> {
    let first_arg = input.argv.pop();

    if let Some(filename) = first_arg {
      println!("filename is {}", filename);

      let contents =
        std::fs::read_to_string(&filename).map_err(|e| format!("Could not read file {}: {}", filename, e))?;

      println!("filename contents is {}", contents);

      let mut payload = PacketMap::default();
      payload.insert("input", &contents);

      let result = input.network.call("inner-schematic", payload).await;
      println!("result ok? {}", result.is_ok());
      let mut packets = result?.drain_port("output").await?;
      println!("packets: {:?}", packets);
      let packet = packets.pop().unwrap();

      let result: u32 = packet.deserialize()?;

      println!("number of bytes: {}", result);

      output.code.done(0)?;
    } else {
      output
        .code
        .done_exception("No argument passed as first argument".to_owned())?;
    }

    Ok(state)
  }
}
