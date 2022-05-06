pub use crate::components::generated::fs_read::*;

pub(crate) type State = ();

#[async_trait::async_trait]
impl wasmflow_sdk::sdk::ephemeral::BatchedComponent for Component {
  type State = State;
  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,
    state: Option<Self::State>,
    _config: Option<Self::Config>,
  ) -> Result<Option<Self::State>, Box<dyn std::error::Error + Send + Sync>> {
    let contents =
      std::fs::read_to_string(&input.filename).map_err(|e| format!("Could not read file {}: {}", input.filename, e))?;
    output.contents.done(contents)?;
    Ok(state)
  }
}
