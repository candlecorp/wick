/// Signature trait for a batched component job.

#[async_trait::async_trait]
pub trait BatchedComponent: super::Component {
  /// For stateful components with a persistent context, this is its type.
  type Context;

  /// The actual work done when a component is invoked.
  async fn job(
    inputs: Self::Inputs,
    outputs: Self::Outputs,
    context: Self::Context,
    config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
