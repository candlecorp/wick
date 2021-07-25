use std::collections::HashMap;

use actix::{
  Addr,
  SyncArbiter,
};
use async_trait::async_trait;
use vino_provider::entity::Entity;
use vino_rpc::error::RpcError;
use vino_rpc::{
  BoxedPacketStream,
  DurationStatistics,
  RpcHandler,
  RpcResult,
  Statistics,
};

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
    let addr = SyncArbiter::start(threads, move || WasmService::new(&module));

    Self { context: addr }
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(
    &self,
    entity: Entity,
    payload: HashMap<String, Vec<u8>>,
  ) -> RpcResult<BoxedPacketStream> {
    let context = self.context.clone();
    let component = entity
      .into_component()
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;
    trace!("Provider running component {}", component);
    let outputs = context
      .send(Call { component, payload })
      .await
      .map_err(|e| RpcError::ProviderError(e.to_string()))??;
    Ok(Box::pin(outputs))
  }

  async fn get_list(&self) -> RpcResult<Vec<vino_rpc::HostedType>> {
    let context = self.context.clone();
    let components = context
      .send(GetComponents {})
      .await
      .map_err(|e| RpcError::ProviderError(e.to_string()))??;
    trace!("Wasm Provider components: {:?}", components);
    Ok(
      components
        .into_iter()
        .map(vino_rpc::HostedType::Component)
        .collect(),
    )
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
  use vino_codec::messagepack::serialize;

  use super::*;

  #[test_env_log::test(actix::test)]
  async fn test_component() -> TestResult<()> {
    let component = crate::helpers::load_wasm_from_file(&PathBuf::from_str(
      "../integration/test-wapc-component/build/test_component_s.wasm",
    )?)?;

    let provider = Provider::new(component, 5);
    let input = "Hello world";

    let job_payload = hashmap! {
      "input".to_owned() => serialize(input)?,
    };

    let mut outputs = provider
      .invoke(Entity::component("validate"), job_payload)
      .await?;
    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.packet);
    let output: String = output.packet.try_into()?;

    println!("output: {:?}", output);
    assert_eq!(output, input);
    Ok(())
  }
}
