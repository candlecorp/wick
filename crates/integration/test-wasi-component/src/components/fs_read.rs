pub use crate::components::generated::fs_read::*;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::ephemeral::BatchedComponent for Component {
  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let contents =
      std::fs::read_to_string(&input.filename).map_err(|e| format!("Could not read file {}: {}", input.filename, e))?;
    output.contents.done(contents)?;
    Ok(())
  }
}
