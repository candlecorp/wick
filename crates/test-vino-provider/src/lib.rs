use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};

use async_trait::async_trait;
use vino_rpc::port::Receiver;
use vino_rpc::{
  RpcHandler,
  RpcResult,
};
mod components;
use anyhow::anyhow;

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
  async fn request(
    &self,
    _inv_id: String,
    component: String,
    payload: HashMap<String, Vec<u8>>,
  ) -> RpcResult<Receiver> {
    let context = self.context.clone();
    let instance = components::get_component(&component);
    match instance {
      Some(instance) => {
        let future = instance.job_wrapper(context, payload);
        Ok(future.await?)
      }
      None => Err(anyhow!("Component '{}' not found", component).into()),
    }
  }
}

#[cfg(test)]
mod tests {
  use futures::prelude::*;
  use maplit::hashmap;
  use vino_codec::messagepack::{
    deserialize,
    serialize,
  };
  use vino_component::{
    v0,
    Output,
  };

  use super::*;

  #[test_env_log::test(tokio::test)]
  async fn request() -> anyhow::Result<()> {
    let provider = Provider::default();
    let input = "some_input";
    let invocation_id = "INVOCATION_ID";
    let job_payload = hashmap! {
      "input".to_string() => serialize(input)?,
    };

    let mut outputs = provider
      .request(
        invocation_id.to_string(),
        "test-component".to_string(),
        job_payload,
      )
      .await
      .expect("request failed");
    let (port_name, output) = outputs.next().await.unwrap();
    println!("Received payload from [{}]", port_name);
    let payload: String = match output {
      Output::V0(v0::Payload::Serializable(payload)) => deserialize(&serialize(payload)?)?,
      _ => None,
    }
    .unwrap();

    println!("outputs: {:?}", payload);
    assert_eq!(payload, "some_input");

    Ok(())
  }
}
