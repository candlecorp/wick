use std::io::{self, BufRead, BufReader};

use wasmflow_sdk::v1::packet::PacketMap;

pub use crate::components::generated::main::*;
use crate::components::*;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::ephemeral::BatchedComponent for Component {
  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("{:?}", input.argv);

    let istty = true; // TODO: Need to detect if STDIN is actually interactive.

    if istty {
      let reader = BufReader::new(io::stdin());
      let input = reader.lines().collect::<Result<Vec<String>, _>>()?.join("\n");

      println!("{}", input);
    }

    output.code.done(0)?;

    Ok(())
  }
}
