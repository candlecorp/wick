use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::RwLock;
use vino_codec::messagepack::serialize;
use vino_provider::native::prelude::*;
use vino_rpc::error::RpcError;
use vino_rpc::{
  RpcHandler,
  RpcResult,
};
use vino_transport::message_transport::stream::BoxedTransportStream;
use vino_transport::message_transport::TransportMap;

use crate::wapc_module::WapcModule;
use crate::wasm_host::WasmHost;
use crate::Error;

#[derive(Debug, Default)]
pub struct Context {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct Provider {
  host: Arc<RwLock<WasmHost>>,
}

impl Provider {
  pub fn try_from_module(module: &WapcModule, threads: usize) -> Result<Self, Error> {
    debug!("WASM:START:{} Threads", threads);

    let host = Arc::new(RwLock::new(WasmHost::try_from(module)?));

    Ok(Self { host })
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, entity: Entity, payload: TransportMap) -> RpcResult<BoxedTransportStream> {
    trace!("WASM:INVOKE:[{}]", entity);
    let component = entity.name();
    let messagepack_map = payload
      .try_into_messagepack_bytes()
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;
    let payload =
      serialize(&messagepack_map).map_err(|e| RpcError::ProviderError(e.to_string()))?;

    let outputs = self.host.write().call(&component, &payload)?;

    Ok(Box::pin(outputs))
  }

  async fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let host = self.host.read();
    let components = host.get_components();

    trace!(
      "WASM:COMPONENTS:[{}]",
      components
        .iter()
        .map(|c| c.name.clone())
        .collect::<Vec<_>>()
        .join(",")
    );

    Ok(
      components
        .iter()
        .cloned()
        .map(HostedType::Component)
        .collect(),
    )
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;
  use std::str::FromStr;

  use anyhow::Result as TestResult;
  use maplit::hashmap;
  use tokio_stream::StreamExt;
  use vino_provider::native::prelude::*;

  use super::*;

  #[test_env_log::test(tokio::test)]
  async fn test_component() -> TestResult<()> {
    let component = crate::helpers::load_wasm_from_file(&PathBuf::from_str(
      "../integration/test-wapc-component/build/test_component_s.wasm",
    )?)
    .await?;

    let provider = Provider::try_from_module(&component, 2)?;
    let input = "Hello world";

    let job_payload = TransportMap::with_map(hashmap! {
      "input".to_owned() => MessageTransport::messagepack(input),
    });

    let mut outputs = provider
      .invoke(Entity::component_direct("validate"), job_payload)
      .await?;
    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let output: String = output.payload.try_into()?;

    println!("output: {:?}", output);
    assert_eq!(output, input);
    Ok(())
  }
}
