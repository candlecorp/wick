mod wick {
  wick_component::wick_import!();
}
use wick::*;

#[async_trait::async_trait(?Send)]
impl AddOperation for Component {
  type Error = anyhow::Error;
  type Outputs = add::Outputs;
  type Config = add::Config;

  async fn add(
    mut left: WickStream<u64>,
    mut right: WickStream<u64>,
    mut outputs: Self::Outputs,
    ctx: Context<Self::Config>,
  ) -> Result<(), Self::Error> {
    while let (Some(Ok(left)), Some(Ok(right))) = (left.next().await, right.next().await) {
      outputs.output.send(&(left + right));
    }
    outputs.output.done();
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl GreetOperation for Component {
  type Error = anyhow::Error;
  type Outputs = greet::Outputs;
  type Config = greet::Config;

  async fn greet(
    mut name: WickStream<String>,
    mut outputs: Self::Outputs,
    ctx: Context<Self::Config>,
  ) -> Result<(), Self::Error> {
    while let (Some(Ok(name))) = (name.next().await) {
      outputs.output.send(&format!("Hello, {}", name));
    }
    outputs.output.done();
    Ok(())
  }
}
