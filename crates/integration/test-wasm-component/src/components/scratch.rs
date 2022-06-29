pub use crate::components::generated::scratch::*;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::ephemeral::BatchedComponent for Component {
  async fn job(
    _input: Self::Inputs,
    _output: Self::Outputs,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;

  #[test_log::test(tokio::test)]
  async fn scratch_test() -> Result<()> {
    let inputs = Inputs {
      age: 32,
      name: "John Doe".to_owned(),
    };

    Ok(())
  }
}
