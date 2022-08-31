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
    let tty = input.is_interactive;
    println!("args: {:?}, interactive: {{ stdin: {}, stdout: {}, stderr: {} }}",
      input.args, tty.stdin, tty.stdout, tty.stderr);

    let isatty = false; // input.is_interactive.stdin
    if !isatty {
      let reader = BufReader::new(io::stdin());
      let input = reader.lines().collect::<Result<Vec<String>, _>>()?.join("\n");

      println!("{}", input);
    }

    output.code.done(0)?;

    Ok(())
  }
}
