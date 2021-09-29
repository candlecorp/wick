use async_trait::async_trait;
use vino_rpc::error::RpcError;
use vino_rpc::{
  RpcHandler,
  RpcResult,
};

use crate::dev::prelude::*;
use crate::network_service::handlers::list_schematics::ListSchematics;

#[derive(Debug, Default)]
struct State {}

#[derive(Clone, Debug)]
pub struct Provider {
  network_id: String,
}

impl Provider {
  #[must_use]
  pub fn new(network_id: String) -> Self {
    Self { network_id }
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, entity: Entity, payload: TransportMap) -> RpcResult<BoxedTransportStream> {
    let addr = NetworkService::for_id(&self.network_id);

    let result: InvocationResponse = addr
      .send(Invocation {
        origin: Entity::Schematic("<system>".to_owned()),
        target: entity,
        msg: payload,
        id: get_uuid(),
        tx_id: get_uuid(),
      })
      .await
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;
    match result.ok() {
      Ok(stream) => Ok(Box::pin(stream)),
      Err(msg) => Err(Box::new(RpcError::ProviderError(format!(
        "Invocation failed: {}",
        msg
      )))),
    }
  }

  async fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let addr = NetworkService::for_id(&self.network_id);
    let result = addr
      .send(ListSchematics {})
      .await
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;
    let schematics = result.map_err(|e| RpcError::ProviderError(e.to_string()))?;
    let hosted_types = schematics.into_iter().map(HostedType::Schematic).collect();
    Ok(hosted_types)
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::test::prelude::{
    assert_eq,
    *,
  };
  type Result<T> = std::result::Result<T, RuntimeError>;

  async fn request_log(provider: &Provider, data: &str) -> Result<String> {
    let job_payload = vec![("input", data)].into();

    let mut outputs = provider
      .invoke(Entity::schematic("simple"), job_payload)
      .await?;
    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let output_data: String = output.payload.try_into()?;

    println!("doc_id: {:?}", output_data);
    assert_eq!(output_data, data);
    Ok(output_data)
  }

  #[test_logger::test(actix_rt::test)]
  async fn test_request_log() -> TestResult<()> {
    let (_, network_id) = init_network_from_yaml("./manifests/v0/simple.yaml").await?;

    let provider = Provider::new(network_id);
    let user_data = "string to log";
    let result = request_log(&provider, user_data).await?;
    print!("Result: {}", result);

    Ok(())
  }

  #[test_logger::test(actix_rt::test)]
  async fn test_list() -> TestResult<()> {
    let (_, network_id) = init_network_from_yaml("./manifests/v0/simple.yaml").await?;
    let provider = Provider::new(network_id);
    let list = provider.get_list().await?;
    println!("components on network : {:?}", list);
    assert_eq!(list.len(), 1);
    Ok(())
  }
}
