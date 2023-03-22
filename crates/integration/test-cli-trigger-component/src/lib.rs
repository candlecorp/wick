use std::io::{self, BufRead, BufReader};

use wasmrs_guest::StreamExt;
use wick_component::packet::CollectionLink;

mod wick {
  wick_component::wick_import!();
}
use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl OpMain for Component {
  async fn main(
    mut args: WickStream<Vec<String>>,
    mut is_interactive: WickStream<Interactive>,
    mut program: WickStream<Option<CollectionLink>>,
    mut outputs: OpMainOutputs,
  ) -> Result<()> {
    while let (Some(Ok(args)), Some(Ok(tty)), Some(Ok(_app))) =
      (args.next().await, is_interactive.next().await, program.next().await)
    {
      // let stream = app.call("hello", PacketStream::default()).await.unwrap();

      println!(
        "args: {:?}, interactive: {{ stdin: {}, stdout: {}, stderr: {} }}",
        args, tty.stdin, tty.stdout, tty.stderr
      );

      let isatty = tty.stdin;
      if !isatty {
        let reader = BufReader::new(io::stdin());
        let lines = reader.lines().collect::<Result<Vec<String>, _>>().unwrap();
        if lines.is_empty() {
          println!("STDIN is non-interactive but had no input.");
        } else {
          println!("<STDIN>{}</STDIN>", lines.join("NL"));
        }
      } else {
        println!("not reading from STDIN, stdin is interactive.");
      }
    }

    let _ = outputs.code.send(&0);
    let _ = outputs.code.done();
    Ok(())
  }
}
