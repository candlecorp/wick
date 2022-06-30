
#[async_trait::async_trait]
/// Signature trait for a batched component job.
pub trait BatchedComponent: super::Component {
  /// The actual work done when a component is invoked.
  async fn job(
    inputs: Self::Inputs,
    outputs: Self::Outputs,
    config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
