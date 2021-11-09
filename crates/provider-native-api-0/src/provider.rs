use vino_entity::Entity;
use vino_provider::native::prelude::*;
use vino_random::Random;
use vino_rpc::error::RpcError;
use vino_rpc::{RpcHandler, RpcResult};

use crate::components::Dispatcher;
use crate::error::NativeError;

#[derive(Clone, Debug)]
pub struct Context {
  #[allow(unused)]
  rng: Random,
}

impl Context {
  pub(crate) fn new(seed: u64) -> Self {
    let rng = Random::from_seed(seed);
    Self { rng }
  }
}

#[derive(Clone, Debug)]
#[must_use]
pub struct Provider {
  context: Context,
}

impl From<NativeError> for Box<RpcError> {
  fn from(e: NativeError) -> Self {
    Box::new(RpcError::ProviderError(e.to_string()))
  }
}

impl Provider {
  pub fn new(seed: u64) -> Self {
    let context = Context::new(seed);
    Self { context }
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, entity: Entity, payload: TransportMap) -> RpcResult<BoxedTransportStream> {
    let context = self.context.clone();
    let component = entity.name();
    let result = Dispatcher::dispatch(&component, context, payload).await;
    let stream = result.map_err(|e| RpcError::ProviderError(e.to_string()))?;

    Ok(Box::pin(stream))
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let signature = crate::components::get_signature();
    Ok(vec![HostedType::Provider(signature)])
  }
}

#[cfg(test)]
mod tests {

  use futures::prelude::*;
  use serde::de::DeserializeOwned;
  use tracing::debug;
  use vino_provider::native::prelude::*;
  use vino_transport::Failure;

  static SEED: u64 = 1000;

  use super::*;
  type Result<T> = std::result::Result<T, NativeError>;

  async fn invoke_one<T>(component: &str, payload: impl Into<TransportMap> + Send) -> Result<T>
  where
    T: DeserializeOwned,
  {
    let transport_map: TransportMap = payload.into();
    println!("TransportMap: {:?}", transport_map);
    let provider = Provider::new(SEED);

    let entity = Entity::component_direct(component);

    let mut outputs = provider.invoke(entity, transport_map).await.unwrap();
    let output = outputs.next().await.unwrap();
    println!(
      "Received payload from port '{}': {:?}",
      output.port, output.payload
    );
    Ok(output.payload.try_into()?)
  }

  async fn invoke_failure(
    component: &str,
    payload: impl Into<TransportMap> + Send,
  ) -> Result<Failure> {
    let transport_map: TransportMap = payload.into();
    println!("TransportMap: {:?}", transport_map);
    let provider = Provider::new(SEED);

    let entity = Entity::component_direct(component);

    let mut outputs = provider.invoke(entity, transport_map).await.unwrap();
    let output = outputs.next().await.unwrap();
    println!(
      "Received payload from port '{}': {:?}",
      output.port, output.payload
    );
    match output.payload {
      MessageTransport::Success(_) => Err("Got success, expected failure".into()),
      MessageTransport::Failure(failure) => Ok(failure),
      MessageTransport::Signal(_) => Err("Got signal, expected failure".into()),
    }
  }

  #[test_logger::test(tokio::test)]
  async fn test_log() -> Result<()> {
    let input = "some_input";
    let job_payload = crate::components::log::Inputs {
      input: input.to_owned(),
    };
    println!("Inputs: {:?}", job_payload);

    let payload: String = invoke_one("log", job_payload).await?;

    println!("outputs: {:?}", payload);
    assert_eq!(payload, "some_input");

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_gate() -> Result<()> {
    let user_data = "Hello world";
    let exception = "Condition is false";
    let inputs = crate::components::gate::Inputs {
      condition: true,
      value: MessageTransport::success(&user_data).into(),
      exception: exception.to_owned(),
    };

    let payload: String = invoke_one("gate", inputs).await?;

    println!("outputs: {:?}", payload);
    assert_eq!(payload, user_data);

    let inputs = crate::components::gate::Inputs {
      condition: false,
      value: MessageTransport::success(&user_data).into(),
      exception: exception.to_owned(),
    };

    let payload = invoke_failure("gate", inputs).await?;

    println!("outputs: {:?}", payload);
    assert_eq!(payload, Failure::Exception(exception.to_owned()));

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn list() -> Result<()> {
    let provider = Provider::new(SEED);
    let signature = crate::components::get_signature();
    let components = signature.components.inner();

    let response = provider.get_list().unwrap();

    debug!("list response : {:?}", response);

    assert_eq!(components.len(), 12);
    assert_eq!(response, vec![HostedType::Provider(signature)]);

    Ok(())
  }
}
