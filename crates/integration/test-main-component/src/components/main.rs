pub use crate::components::generated::main::*;

pub(crate) type State = ();

#[async_trait::async_trait]
impl wasmflow_sdk::v1::ephemeral::BatchedComponent for Component {
  type State = State;
  async fn job(
    mut input: Self::Inputs,
    output: Self::Outputs,
    state: Option<Self::State>,
    _config: Option<Self::Config>,
  ) -> Result<Option<Self::State>, Box<dyn std::error::Error + Send + Sync>> {
    let first_arg = input.argv.pop();
    if let Some(filename) = first_arg {
      println!("filename is {}", filename);
      let contents =
        std::fs::read_to_string(&filename).map_err(|e| format!("Could not read file {}: {}", filename, e))?;
      println!("filename contents is {}", contents);
      let code = if !contents.is_empty() { 0 } else { 1 };
      output.code.done(code)?;
    } else {
      output
        .code
        .done_exception("No argument passed as first argument".to_owned())?;
    }

    Ok(state)
  }
}
