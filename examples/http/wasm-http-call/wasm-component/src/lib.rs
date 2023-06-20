mod wick {
  wick_component::wick_import!();
}
use wick::*;

#[async_trait::async_trait(?Send)]
impl RequestOperation for Component {
  type Error = anyhow::Error;
  type Outputs = request::Outputs;
  type Config = request::Config;

  async fn request(
    mut id: WickStream<String>,
    mut name: WickStream<String>,
    mut outputs: Self::Outputs,
    ctx: Context<Self::Config>,
  ) -> Result<(), Self::Error> {
    while let (Some(id), Some(name)) = (id.next().await, name.next().await) {
      let id = propagate_if_error!(id, outputs, continue);
      let name = propagate_if_error!(name, outputs, continue);

      // `ctx.provided()` returns the external components that have
      // been provided to this component, like `client` which we defined
      // in the `require` section of our component.wick.
      let client = &ctx.provided().client;

      // `post_op` is a method on the `client` component. It takes two inputs and produces
      // two outputs. In wick, all inputs and outputs are independent streams.
      let (mut response, mut body) = client.post_op(
        PostOpConfig {
          message: "From wasm".to_owned(),
        },
        once(id),
        once(name),
      )?;

      if let Some(response) = response.next().await {
        let _response = propagate_if_error!(response, outputs, continue);
        // Here's where we can check response headers, status, etc.
      }

      if let Some(body) = body.next().await {
        let body = propagate_if_error!(body, outputs, continue);
        // Here's the
        outputs.output.send(&body);
      }
    }
    outputs.output.done();
    Ok(())
  }
}
