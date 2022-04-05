use vino_provider::native::prelude::*;
use vino_random::{Random, Seed};
use vino_rpc::error::RpcError;
use vino_rpc::{RpcHandler, RpcResult};
use vino_transport::Invocation;

use crate::components::Dispatcher;
use crate::error::NativeError;

#[derive(Debug)]
pub struct Context {
  rng: Random,
}

impl Clone for Context {
  fn clone(&self) -> Self {
    Self {
      rng: Random::from_seed(self.rng.seed()),
    }
  }
}

impl Context {
  pub(crate) fn new(seed: Seed) -> Self {
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
  pub fn new(seed: Seed) -> Self {
    let context = Context::new(seed);
    Self { context }
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, invocation: Invocation) -> RpcResult<BoxedTransportStream> {
    let context = self.context.clone();
    let component = invocation.target.name();
    let result = Dispatcher::dispatch(component, context, invocation.payload).await;
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

  static SEED: u64 = 1000;

  use super::*;
  type Result<T> = std::result::Result<T, NativeError>;

  async fn invoke_one<T>(component: &str, payload: impl Into<TransportMap> + Send) -> Result<T>
  where
    T: DeserializeOwned,
  {
    let transport_map: TransportMap = payload.into();
    println!("TransportMap: {:?}", transport_map);
    let provider = Provider::new(Seed::unsafe_new(SEED));

    let entity = Entity::local_component(component);
    let invocation = Invocation::new_test(file!(), entity, transport_map, None);

    let mut outputs = provider.invoke(invocation).await.unwrap();
    let output = outputs.next().await.unwrap();
    println!("Received payload from port '{}': {:?}", output.port, output.payload);
    Ok(output.payload.deserialize()?)
  }

  #[test_logger::test(tokio::test)]
  async fn test_log() -> Result<()> {
    let input = "some_input";
    let job_payload = crate::components::core::log::Inputs {
      input: input.to_owned(),
    };
    println!("Inputs: {:?}", job_payload);

    let payload: String = invoke_one("core::log", job_payload).await?;

    println!("outputs: {:?}", payload);
    assert_eq!(payload, "some_input");

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn list() -> Result<()> {
    let provider = Provider::new(Seed::unsafe_new(SEED));
    let signature = crate::components::get_signature();
    let components = signature.components.inner();

    let response = provider.get_list().unwrap();

    debug!("list response : {:?}", response);

    assert_eq!(components.len(), 9);
    assert_eq!(response, vec![HostedType::Provider(signature)]);

    Ok(())
  }
}
