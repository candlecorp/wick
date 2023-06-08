mod wick {
  wick_component::wick_import!();
}
use wick::*;

#[async_trait::async_trait(?Send)]
impl MainOperation for Component {
  type Error = anyhow::Error;
  type Outputs = main::Outputs;
  type Config = main::Config;

  async fn main(
    mut args: WickStream<u64>,
    mut is_interactive: WickStream<u64>,
    mut outputs: Self::Outputs,
    ctx: Context<Self::Config>,
  ) -> Result<(), Self::Error> {
    while let (Some(args), Some(is_interactive)) = (args.next().await, is_interactive.next().await) {
      let args = propagate_if_error!(args, outputs, continue);
      let interactive = propagate_if_error!(is_interactive, outputs, continue);
      let interface = ctx.provided().component.component()
      outputs.output.send(&(left + right));
    }
    outputs.output.done();
    Ok(())
  }
}
