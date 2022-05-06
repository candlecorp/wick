pub use crate::components::generated::scratch::*;

pub(crate) type State = ();

#[async_trait::async_trait]
impl wasmflow_sdk::sdk::ephemeral::BatchedComponent for Component {
  type State = State;
  async fn job(
    _input: Self::Inputs,
    _output: Self::Outputs,
    state: Option<Self::State>,
    _config: Option<Self::Config>,
  ) -> Result<Option<Self::State>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(state)
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
