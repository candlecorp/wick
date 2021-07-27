use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::Mutex;
use vino_rpc::error::RpcError;
use vino_rpc::port::PacketWrapper;
use vino_rpc::{
  BoxedPacketStream,
  DurationStatistics,
  HostedType,
  RpcHandler,
  RpcResult,
  Statistics,
};

use crate::dev::prelude::*;
use crate::network_service::handlers::list_schematics::ListSchematics;

#[derive(Debug, Default)]
struct State {
  documents: HashMap<String, String>,
  collections: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct Provider {
  network_id: String,
  context: Arc<Mutex<State>>,
}

impl Provider {
  #[must_use]
  pub fn new(network_id: String) -> Self {
    Self {
      network_id,
      context: Arc::new(Mutex::new(State::default())),
    }
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(
    &self,
    entity: Entity,
    payload: HashMap<String, Vec<u8>>,
  ) -> RpcResult<BoxedPacketStream> {
    let addr = NetworkService::for_id(&self.network_id);
    let result: InvocationResponse = addr
      .send(Invocation {
        origin: Entity::Schematic("<system>".to_owned()),
        target: entity,
        msg: MessageTransport::MultiBytes(payload),
        id: get_uuid(),
        tx_id: get_uuid(),
        network_id: get_uuid(),
      })
      .await
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;
    match result {
      InvocationResponse::Stream { rx, .. } => Ok(Box::pin(rx.map(|output| PacketWrapper {
        port: output.port,
        packet: output.payload,
      }))),
      InvocationResponse::Error { msg, .. } => Err(Box::new(RpcError::ProviderError(format!(
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
  use maplit::hashmap;

  use super::*;
  use crate::test::prelude::{
    assert_eq,
    *,
  };
  type Result<T> = std::result::Result<T, RuntimeError>;

  async fn request_log(provider: &Provider, data: &str) -> Result<String> {
    let job_payload = hashmap! {
      "input".to_owned() => mp_serialize(data)?,
    };

    let mut outputs = provider
      .invoke(Entity::schematic("simple"), job_payload)
      .await?;
    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.packet);
    let output_data: String = output.packet.try_into()?;

    println!("doc_id: {:?}", output_data);
    assert_eq!(output_data, data);
    Ok(output_data)
  }

  #[test_env_log::test(actix_rt::test)]
  async fn test_request_log() -> TestResult<()> {
    let (_, network_id) = init_network_from_yaml("./manifests/v0/network/simple.yaml").await?;

    let provider = Provider::new(network_id);
    let user_data = "string to log";
    let result = request_log(&provider, user_data).await?;
    print!("Result: {}", result);

    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn test_list() -> TestResult<()> {
    let (_, network_id) = init_network_from_yaml("./manifests/v0/network/simple.yaml").await?;
    let provider = Provider::new(network_id);
    let list = provider.get_list().await?;
    println!("components on network : {:?}", list);
    assert_eq!(list.len(), 1);
    Ok(())
  }
}
