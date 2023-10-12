#[cfg(feature = "localgen")]
mod generated;
#[cfg(feature = "localgen")]
use generated as wick;
#[cfg(not(feature = "localgen"))]
mod wick {
  #![allow(unused_imports, missing_debug_implementations, clippy::needless_pass_by_value)]
  wick_component::wick_import!();
}

use provided::client_component::post_op;
use wick::*;

#[async_trait::async_trait(?Send)]
impl request::Operation for Component {
  type Error = anyhow::Error;
  type Inputs = request::Inputs;
  type Outputs = request::Outputs;
  type Config = request::Config;

  async fn request(
    inputs: Self::Inputs,
    mut outputs: Self::Outputs,
    ctx: Context<Self::Config>,
  ) -> Result<(), Self::Error> {
    let Self::Inputs { mut id, mut name } = inputs;
    while let (Some(id), Some(name)) = (id.next().await, name.next().await) {
      let id = propagate_if_error!(id.decode(), outputs, continue);
      let name = propagate_if_error!(name.decode(), outputs, continue);

      // `ctx.provided()` returns the external components that have
      // been provided to this component, like `client` which we defined
      // in the `require` section of our component.wick.
      let client = &ctx.provided().client;

      // `post_op` is a method on the `client` component. It takes two inputs and produces
      // two outputs. In wick, all inputs and outputs are independent streams.
      let mut response = client.post_op(
        post_op::Config {
          message: "From wasm".to_owned(),
        },
        post_op::Request { id, name },
      )?;

      if let Some(response) = response.response.next().await {
        let _response = propagate_if_error!(response.decode(), outputs, continue);
        // Here's where we can check response headers, status, etc.
      }

      if let Some(body) = response.body.next().await {
        let body = propagate_if_error!(body.decode(), outputs, continue);
        // Here's the
        outputs.output.send(&body);
      }
    }
    outputs.output.done();
    Ok(())
  }
}
