use async_trait::async_trait;
use tracing::Instrument;
use uuid::Uuid;
use vino_rpc::error::RpcError;
use vino_rpc::{RpcHandler, RpcResult};

use crate::dev::prelude::*;

#[derive(Debug, Default)]
struct State {}

#[derive(Clone, Copy, Debug)]
pub struct Provider {
  network_id: Uuid,
}

impl Provider {
  #[must_use]
  pub fn new(network_id: Uuid) -> Self {
    Self { network_id }
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, invocation: Invocation) -> RpcResult<BoxedTransportStream> {
    let target_url = invocation.target_url();

    let span = debug_span!(
      "invoke",
      network_id = %self.network_id,
      target =  %invocation.target
    );

    let network = NetworkService::for_id(&self.network_id)
      .ok_or_else(|| Box::new(RpcError::ProviderError(format!("Network '{}' not found", target_url))))?;

    trace!(target = %target_url, "invoking");

    let result: InvocationResponse = network
      .invoke(invocation)
      .map_err(|e| RpcError::ProviderError(e.to_string()))?
      .instrument(span)
      .await
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;

    match result.ok() {
      Ok(stream) => Ok(Box::pin(stream)),
      Err(msg) => Err(Box::new(RpcError::ProviderError(format!("Invocation failed: {}", msg)))),
    }
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let addr = NetworkService::for_id(&self.network_id).ok_or_else(|| {
      Box::new(RpcError::ProviderError(format!(
        "Network '{}' not found",
        self.network_id
      )))
    })?;
    let signature = addr
      .get_signature()
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;
    Ok(vec![HostedType::Provider(signature)])
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::test::prelude::{assert_eq, *};
  type Result<T> = anyhow::Result<T>;

  async fn request_log(provider: &Provider, data: &str) -> Result<String> {
    let job_payload = vec![("input", data)].into();

    let invocation = Invocation::new_test(file!(), Entity::local_component("simple"), job_payload, None);
    let mut outputs = provider.invoke(invocation).await?;
    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let output_data: String = output.payload.deserialize()?;

    println!("doc_id: {:?}", output_data);
    assert_eq!(output_data, data);
    Ok(output_data)
  }

  #[test_logger::test(tokio::test)]
  async fn test_request_log() -> TestResult<()> {
    let (_, network_id) = init_network_from_yaml("./manifests/v0/simple.yaml").await?;

    let provider = Provider::new(network_id);
    let user_data = "string to log";
    let result = request_log(&provider, user_data).await?;
    print!("Result: {}", result);

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_list() -> TestResult<()> {
    let (_, network_id) = init_network_from_yaml("./manifests/v0/simple.yaml").await?;
    let provider = Provider::new(network_id);
    let list = provider.get_list()?;
    println!("components on network : {:?}", list);
    assert_eq!(list.len(), 1);
    Ok(())
  }
}
