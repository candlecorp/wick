use vino_provider::native::prelude::*;
use vino_rpc::error::RpcError;
use vino_rpc::{RpcHandler, RpcResult};

use self::components::Dispatcher;
pub mod components;

#[macro_use]
extern crate tracing;

#[derive(Clone)]
pub struct Context {}

#[derive(Clone)]
pub struct Provider {
  context: Context,
}

impl Provider {
  pub fn default() -> Self {
    Self {
      context: Context {},
    }
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, entity: Entity, payload: TransportMap) -> RpcResult<BoxedTransportStream> {
    trace!("TEST_PROVIDER:INVOKE[{}]", entity);
    let context = self.context.clone();
    let component = entity.name();
    let result = Dispatcher::dispatch(&component, context, payload)
      .await
      .map_err(|e| RpcError::ProviderError(e.to_string()));
    trace!("TEST_PROVIDER:INVOKE[{}]:RESULT:{:?}", entity, result);
    let stream = result?;

    Ok(Box::pin(stream))
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    trace!("TEST_PROVIDER:GET_LIST");
    let signature = components::get_signature();
    Ok(vec![HostedType::Provider(signature)])
  }
}

#[cfg(test)]
mod tests {

  use futures::prelude::*;
  use tracing::*;
  use vino_provider::native::prelude::*;

  use super::*;

  #[test_logger::test(tokio::test)]
  async fn request() -> anyhow::Result<()> {
    let provider = Provider::default();
    let input = "some_input";
    let job_payload = TransportMap::from_map(hashmap! {
      "input".to_string() => MessageTransport::messagepack(input),
    });

    let entity = Entity::component_direct("test-component");

    let mut outputs = provider.invoke(entity, job_payload).await?;
    let output = outputs.next().await.unwrap();
    println!("Received payload from [{}]", output.port);
    let payload: String = output.payload.try_into().unwrap();

    println!("outputs: {:?}", payload);
    assert_eq!(payload, "TEST: some_input");

    Ok(())
  }

  #[test_logger::test(tokio::test)]
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
}
