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
use provided::baseline::power;
use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl main::Operation for Component {
  type Error = anyhow::Error;
  type Inputs = main::Inputs;
  type Outputs = main::Outputs;
  type Config = main::Config;

  async fn main(
    inputs: Self::Inputs,
    mut outputs: Self::Outputs,
    ctx: Context<Self::Config>,
  ) -> Result<(), Self::Error> {
    let Self::Inputs {
      mut args,
      mut interactive,
    } = inputs;
    while let (Some(args), Some(tty)) = (args.next().await, interactive.next().await) {
      let args = args.decode()?;
      let tty = tty.decode()?;
      println!(
        "args: {:?}, interactive: {{ stdin: {}, stdout: {}, stderr: {} }}",
        args, tty.stdin, tty.stdout, tty.stderr
      );

      let mut provided_component_result = ctx
        .provided()
        .baseline
        .power(power::Config { exponent: 3 }, power::Request { input: 2 })?;

      if let Some(result) = provided_component_result.output.next().await {
        println!("Got result for provided component: {}", result.decode()?);
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
