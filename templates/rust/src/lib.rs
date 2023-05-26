use wasmrs_guest::*;
mod wick {
  wick_component::wick_import!();
}
use wick::*;

#[async_trait::async_trait(?Send)]
impl OpAdd for Component {
  async fn add(
    mut left: WickStream<u64>,
    mut right: WickStream<u64>,
    mut outputs: OpAddOutputs,
    ctx: Context<OpAddConfig>,
  ) -> wick::Result<()> {
    while let (Some(Ok(left)), Some(Ok(right))) = (left.next().await, right.next().await) {
      outputs.output.send(&(left + right));
    }
    outputs.output.done();
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl OpGreet for Component {
  async fn greet(
    mut name: WickStream<String>,
    mut outputs: OpGreetOutputs,
    ctx: Context<OpGreetConfig>,
  ) -> wick::Result<()> {
    while let (Some(Ok(name))) = (name.next().await) {
      outputs.output.send(&format!("Hello, {}", name));
    }
    outputs.output.done();
    Ok(())
  }
}
