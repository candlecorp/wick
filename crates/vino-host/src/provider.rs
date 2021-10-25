use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use vino_provider::native::prelude::*;
use vino_rpc::error::RpcError;
use vino_rpc::{
  RpcHandler,
  RpcResult,
};
use vino_runtime::core_data::InitData;
use vino_transport::message_transport::stream::BoxedTransportStream;
use vino_transport::TransportMap;

use crate::Host;

#[derive(Debug, Default)]
pub struct Context {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct Provider {
  host: Arc<Host>,
}

impl Provider {}

impl From<Host> for Provider {
  fn from(host: Host) -> Self {
    Self {
      host: Arc::new(host),
    }
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, entity: Entity, payload: TransportMap) -> RpcResult<BoxedTransportStream> {
    trace!("HOST:INVOKE:[{}]", entity);
    let component = entity.name();

    let init_data: InitData = payload
      .get_config()
      .clone()
      .map(|c| c.into())
      .unwrap_or_default();

    let outputs = self
      .host
      .request(&component, payload, Some(init_data))
      .await
      .map_err(RpcError::boxed)?;

    Ok(Box::pin(outputs))
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let provider: ProviderSignature = self.host.get_signature().map_err(RpcError::boxed)?;

    Ok(vec![HostedType::Provider(provider)])
  }
}

#[cfg(test)]
mod tests {
  use std::convert::TryFrom;

  use anyhow::Result as TestResult;
  use tokio_stream::StreamExt;
  use vino_provider::native::prelude::*;

  use super::*;
  use crate::HostBuilder;

  #[test_logger::test(tokio::test)]
  async fn test_component() -> TestResult<()> {
    let builder = HostBuilder::try_from("./manifests/logger.yaml")?;
    let mut host = builder.build();
    host.start().await?;
    let provider: Provider = host.into();
    let input = "Hello world";

    let job_payload = TransportMap::from(vec![("input", input)]);

    let mut outputs = provider
      .invoke(Entity::component_direct("logger"), job_payload)
      .await?;
    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let output: String = output.payload.try_into()?;

    println!("output: {:?}", output);
    assert_eq!(output, input);
    Ok(())
  }
}
