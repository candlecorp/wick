use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};

use async_trait::async_trait;
use vino_entity::Entity;
use vino_rpc::error::RpcError;
use vino_rpc::{
  BoxedPacketStream,
  DurationStatistics,
  RpcHandler,
  RpcResult,
  Statistics,
};
mod components;
pub(crate) mod generated;

pub(crate) struct State {}

#[derive(Clone)]
pub struct Provider {
  context: Arc<Mutex<State>>,
}

impl Provider {
  pub fn default() -> Self {
    Self {
      context: Arc::new(Mutex::new(State {})),
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
    let context = self.context.clone();
    let component = entity
      .into_component()
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;
    let instance = generated::get_component(&component);
    match instance {
      Some(instance) => {
        let future = instance.job_wrapper(context, payload);
        Ok(Box::pin(
          future
            .await
            .map_err(|e| RpcError::ProviderError(e.to_string()))?,
        ))
      }
      None => Err(Box::new(RpcError::ProviderError(format!(
        "Component '{}' not found",
        component
      )))),
    }
  }

  async fn get_list(&self) -> RpcResult<Vec<vino_rpc::HostedType>> {
    let components = generated::get_all_components();
    Ok(
      components
        .into_iter()
        .map(vino_rpc::HostedType::Component)
        .collect(),
    )
  }

  async fn get_stats(&self, id: Option<String>) -> RpcResult<Vec<vino_rpc::Statistics>> {
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

  use futures::prelude::*;
  use maplit::hashmap;
  use tracing::debug;
  use vino_codec::messagepack::{
    deserialize,
    serialize,
  };
  use vino_component::{
    v0,
    Packet,
  };
  use vino_rpc::{
    ComponentSignature,
    HostedType,
    PortSignature,
  };

  use super::*;

  #[test_env_log::test(tokio::test)]
  async fn request() -> anyhow::Result<()> {
    let provider = Provider::default();
    let input = "some_input";
    let job_payload = hashmap! {
      "input".to_string() => serialize(input)?,
    };

    let entity = Entity::component("test-component");

    let mut outputs = provider
      .invoke(entity, job_payload)
      .await
      .expect("request failed");
    let output = outputs.next().await.unwrap();
    println!("Received payload from [{}]", output.port);
    let payload: String = match output.packet {
      Packet::V0(v0::Payload::MessagePack(payload)) => deserialize(&payload)?,
      _ => None,
    }
    .unwrap();

    println!("outputs: {:?}", payload);
    assert_eq!(payload, "TEST: some_input");

    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn list() -> anyhow::Result<()> {
    let provider = Provider::default();

    let response = provider.get_list().await.expect("request failed");

    debug!("list response : {:?}", response);

    assert_eq!(response.len(), 1);
    assert_eq!(
      response[0],
      HostedType::Component(ComponentSignature {
        name: "test-component".to_string(),
        inputs: vec![PortSignature {
          name: "input".to_string(),
          type_string: "string".to_string()
        }],
        outputs: vec![PortSignature {
          name: "output".to_string(),
          type_string: "string".to_string()
        }]
      })
    );

    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn statistics() -> anyhow::Result<()> {
    let provider = Provider::default();

    let response = provider.get_stats(None).await.expect("request failed");

    debug!("statistics response : {:?}", response);

    assert_eq!(response.len(), 1);

    Ok(())
  }
}
