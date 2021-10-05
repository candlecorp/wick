use vino_provider::native::prelude::*;
use vino_rpc::error::RpcError;
use vino_rpc::{
  RpcHandler,
  RpcResult,
};

use self::generated::Dispatcher;
mod components;
pub(crate) mod generated;

#[derive(Clone)]
pub(crate) struct Context {}

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
    let context = self.context.clone();
    let component = entity.name();
    let stream = Dispatcher::dispatch(&component, context, payload)
      .await
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;

    Ok(Box::pin(stream))
  }

  async fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let signature = generated::get_signature();
    Ok(vec![HostedType::Provider(signature)])
  }
}

#[cfg(test)]
mod tests {

  use futures::prelude::*;
  use log::*;
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
