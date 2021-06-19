use std::sync::{
  Arc,
  Mutex,
};

use async_trait::async_trait;
use vino_provider::error::ProviderError;
use vino_provider::port::Receiver;
use vino_provider::{
  ProviderHandler,
  Result,
};
mod components;

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
impl ProviderHandler for Provider {
  async fn request(
    &self,
    _inv_id: String,
    component: String,
    payload: Vec<u8>,
  ) -> Result<Receiver> {
    let context = self.context.clone();
    let instance = components::get_component(&component);
    match instance {
      Some(instance) => {
        let future = instance.job_wrapper(context, &payload);
        future.await
      }
      None => Err(ProviderError::ComponentNotFound(component.to_string())),
    }
  }
}

#[cfg(test)]
mod tests {
  use futures::prelude::*;
  use maplit::hashmap;
  use vino_guest::OutputPayload;
  use vino_runtime::{
    deserialize,
    serialize,
  };

  use super::*;

  #[test_env_log::test(tokio::test)]
  async fn request() -> Result<()> {
    let provider = Provider::default();
    let input = "some_input";
    let invocation_id = "INVOCATION_ID";
    let job_payload = hashmap! {
      "input".to_string() => serialize(input)?,
    };
    let payload = serialize(job_payload)?;

    let mut outputs = provider
      .request(
        invocation_id.to_string(),
        "vino::test::provider".to_string(),
        payload,
      )
      .await?;
    let (port_name, payload) = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", port_name, payload);
    let payload = match payload {
      OutputPayload::MessagePack(payload) => deserialize::<String>(&payload).ok(),
      _ => None,
    }
    .unwrap();

    println!("outputs: {:?}", payload);
    assert_eq!(payload, "some_input");

    Ok(())
  }
}
