use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use wasmflow_rpc::error::RpcError;
use wasmflow_rpc::{RpcHandler, RpcResult};
use wasmflow_transport::TransportStream;
use wasmflow_interface::*;
use wasmflow_invocation::Invocation;

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
    Self { host: Arc::new(host) }
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, invocation: Invocation) -> RpcResult<TransportStream> {
    let outputs = self.host.invoke(invocation).await.map_err(RpcError::boxed)?;

    Ok(outputs)
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
  use wasmflow_entity::Entity;
  use wasmflow_runtime::prelude::packet::PacketMap;

  use super::*;
  use crate::HostBuilder;

  #[test_logger::test(tokio::test)]
  async fn test_component() -> TestResult<()> {
    let builder = HostBuilder::try_from("./manifests/logger.yaml")?;
    let mut host = builder.build();
    host.start(Some(0)).await?;
    let provider: Provider = host.into();
    let input = "Hello world";

    let job_payload = PacketMap::from(vec![("input", input)]);

    let invocation = Invocation::new_test(file!(), Entity::local("logger"), job_payload, None);
    let mut outputs = provider.invoke(invocation).await?;
    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let output: String = output.payload.deserialize()?;

    println!("output: {:?}", output);
    assert_eq!(output, input);
    Ok(())
  }
}
