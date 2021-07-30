use std::collections::HashMap;

use actix::{
  Addr,
  SyncArbiter,
};
use async_trait::async_trait;
use vino_provider::native::prelude::*;
use vino_rpc::error::RpcError;
use vino_rpc::{
  BoxedTransportStream,
  DurationStatistics,
  RpcHandler,
  RpcResult,
  Statistics,
};
use vino_transport::message_transport::TransportMap;

use crate::wapc_module::WapcModule;
use crate::wasm_service::{
  Call,
  GetComponents,
  WasmService,
};

#[derive(Debug, Default)]
pub struct State {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct Provider {
  context: Addr<WasmService>,
}

impl Provider {
  #[must_use]
  pub fn new(module: WapcModule, threads: usize) -> Self {
    debug!("PRV:WASM:START:{} Threads", threads);
    let addr = SyncArbiter::start(threads, move || WasmService::new(&module));

    Self { context: addr }
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, entity: Entity, payload: TransportMap) -> RpcResult<BoxedTransportStream> {
    trace!("PRV:WASM:INVOKE:[{}]", entity);
    let context = self.context.clone();
    let component = entity
      .into_component()
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;
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
      "PRV:WASM:COMPONENTS:[{}]",
      components
        .iter()
        .map(|c| c.name.clone())
        .collect::<Vec<_>>()
        .join(",")
    );

    Ok(components.into_iter().map(HostedType::Component).collect())
  }

  async fn get_stats(&self, id: Option<String>) -> RpcResult<Vec<Statistics>> {
    // TODO Dummy implementation
    if id.is_some() {
      Ok(vec![Statistics {
        num_calls: 1,
        execution_duration: DurationStatistics {
          max_time: 0,
          min_time: 0,
          average: 0,
        },
      }])
    } else {
      Ok(vec![Statistics {
        num_calls: 0,
        execution_duration: DurationStatistics {
          max_time: 0,
          min_time: 0,
          average: 0,
        },
      }])
    }
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;
  use std::str::FromStr;

  use anyhow::Result as TestResult;
  use futures::prelude::*;
  use maplit::hashmap;
  use vino_provider::native::prelude::*;

  use super::*;

  #[test_env_log::test(actix::test)]
  async fn test_component() -> TestResult<()> {
    let component = crate::helpers::load_wasm_from_file(&PathBuf::from_str(
      "../integration/test-wapc-component/build/test_component_s.wasm",
    )?)?;

    let provider = Provider::new(component, 2);
    let input = "Hello world";

    let job_payload = TransportMap::with_map(hashmap! {
      "input".to_owned() => MessageTransport::messagepack(input),
    });

    let mut outputs = provider
      .invoke(Entity::component("validate"), job_payload)
      .await?;
    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let output: String = output.payload.try_into()?;

    println!("output: {:?}", output);
    assert_eq!(output, input);
    Ok(())
  }
}
