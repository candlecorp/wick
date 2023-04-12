use std::io::{self, BufRead, BufReader};

use wasmrs_guest::StreamExt;
use wick_component::packet as wick_packet;
use wick_component::packet::{packet_stream, CollectionLink};
mod generated;
use generated as wick;
// mod wick {
//   wick_component::wick_import!();
// }
use wick::*;
mod manual;
use manual::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl OpMain for Component {
  async fn main(
    mut args: WickStream<Vec<String>>,
    mut is_interactive: WickStream<Interactive>,
    mut outputs: OpMainOutputs,
  ) -> Result<()> {
    while let (Some(Ok(args)), Some(Ok(tty))) = (args.next().await, is_interactive.next().await) {
      println!(
        "args: {:?}, interactive: {{ stdin: {}, stdout: {}, stderr: {} }}",
        args, tty.stdin, tty.stdout, tty.stderr
      );
      let id: u32 = args.get(0).unwrap_or(&"0".to_string()).parse().unwrap();
      let packets = packet_stream!(("id", id));
      let response = get_config().db.call("get_user", packets);
    }

    let _ = outputs.code.send(&0);
    let _ = outputs.code.done();
    Ok(())
  }
}
