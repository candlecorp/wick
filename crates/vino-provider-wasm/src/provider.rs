use std::collections::HashMap;
use std::convert::TryFrom;

use actix::{
  Addr,
  SyncArbiter,
};
use async_trait::async_trait;
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
use crate::wasm_service::{
  Call,
  GetComponents,
  WasmService,
};
use crate::Error;

#[derive(Debug, Default)]
pub struct Context {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct Provider {
  context: Addr<WasmService>,
}

impl Provider {
  pub fn try_from_module(module: WapcModule, threads: usize) -> Result<Self, Error> {
    debug!("WASM:START:{} Threads", threads);

    let addr = SyncArbiter::start(threads, move || {
      let host = WasmHost::try_from(&module).unwrap();
      WasmService::new(host)
    });

    Ok(Self { context: addr })
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, entity: Entity, payload: TransportMap) -> RpcResult<BoxedTransportStream> {
    trace!("WASM:INVOKE:[{}]", entity);
    let context = self.context.clone();
    let component = entity.name();
    let messagepack_map = payload
      .try_into_messagepack_bytes()
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;

    let outputs = context
      .send(Call {
        component,
        payload: messagepack_map,
      })
      .await
      .map_err(|e| RpcError::ProviderError(e.to_string()))??;
    Ok(Box::pin(outputs))
  }

  async fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let context = self.context.clone();
    let components = context
      .send(GetComponents {})
      .await
      .map_err(|e| RpcError::ProviderError(e.to_string()))??;

    trace!(
      "WASM:COMPONENTS:[{}]",
      components
        .iter()
        .map(|c| c.name.clone())
        .collect::<Vec<_>>()
        .join(",")
    );

    Ok(components.into_iter().map(HostedType::Component).collect())
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

  #[test_env_log::test(actix::test)]
  async fn test_component() -> TestResult<()> {
    let component = crate::helpers::load_wasm_from_file(&PathBuf::from_str(
      "../integration/test-wapc-component/build/test_component_s.wasm",
    )?)
    .await?;

    let provider = Provider::try_from_module(component, 2)?;
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
