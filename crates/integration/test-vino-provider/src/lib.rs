use std::sync::{
  Arc,
  Mutex,
};

use vino_provider::native::prelude::*;
use vino_rpc::error::RpcError;
use vino_rpc::{
  BoxedTransportStream,
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
  async fn invoke(&self, entity: Entity, payload: TransportMap) -> RpcResult<BoxedTransportStream> {
    let context = self.context.clone();
    let component = entity
      .into_component()
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;
    let instance = generated::get_component(&component);
    match instance {
      Some(instance) => {
        let future = instance.execute(context, payload);
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

  async fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let components = generated::get_all_components();
    Ok(components.into_iter().map(HostedType::Component).collect())
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
  use log::*;
  use maplit::hashmap;
  use vino_provider::native::prelude::*;

  use super::*;

  #[test_env_log::test(tokio::test)]
  async fn request() -> anyhow::Result<()> {
    let provider = Provider::default();
    let input = "some_input";
    let job_payload = TransportMap::with_map(hashmap! {
      "input".to_string() => MessageTransport::messagepack(input),
    });

    let entity = Entity::component("test-component");

    let mut outputs = provider.invoke(entity, job_payload).await?;
    let output = outputs.next().await.unwrap();
    println!("Received payload from [{}]", output.port);
    let payload: String = output.payload.try_into().unwrap();

    println!("outputs: {:?}", payload);
    assert_eq!(payload, "TEST: some_input");

    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn list() -> anyhow::Result<()> {
    let provider = Provider::default();

    let response = provider.get_list().await?;

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

    let response = provider.get_stats(None).await?;

    debug!("statistics response : {:?}", response);

    assert_eq!(response.len(), 1);

    Ok(())
  }
}
