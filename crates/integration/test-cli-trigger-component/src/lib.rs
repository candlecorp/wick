use std::io::{self, BufRead, BufReader};

#[cfg(feature = "localgen")]
mod generated;
#[cfg(feature = "localgen")]
use generated as wick;
#[cfg(not(feature = "localgen"))]
mod wick {
  #![allow(unused_imports, missing_debug_implementations, clippy::needless_pass_by_value)]
  wick_component::wick_import!();
}
use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl MainOperation for Component {
  type Error = anyhow::Error;
  type Outputs = main::Outputs;
  type Config = main::Config;

  async fn main(
    mut args: WickStream<Vec<String>>,
    mut is_interactive: WickStream<types::cli::Interactive>,
    mut outputs: Self::Outputs,
    ctx: Context<Self::Config>,
  ) -> Result<(), Self::Error> {
    while let (Some(Ok(args)), Some(Ok(tty))) = (args.next().await, is_interactive.next().await) {
      println!(
        "args: {:?}, interactive: {{ stdin: {}, stdout: {}, stderr: {} }}",
        args, tty.stdin, tty.stdout, tty.stderr
      );

      let mut provided_component_result = ctx.provided().baseline.power(PowerConfig { exponent: 3 }, once(2))?;

      if let Some(Ok(result)) = provided_component_result.next().await {
        println!("Got result for provided component: {}", result);
      } else {
        println!("Got no result for provided component.");
      }

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
