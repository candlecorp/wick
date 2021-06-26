use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};

use async_trait::async_trait;
use vino_rpc::port::Receiver;
use vino_rpc::{
  ExecutionStatistics,
  RpcHandler,
  RpcResult,
  Statistics,
};
mod components;
pub mod error;
pub type Result<T> = std::result::Result<T, error::NativeError>;

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
      None => Err(format!("Could not find component: {}", component).into()),
    }
  }

  async fn list_registered(&self) -> RpcResult<Vec<vino_rpc::HostedType>> {
    let components = components::get_all_components();
    Ok(
      components
        .into_iter()
        .map(vino_rpc::HostedType::Component)
        .collect(),
    )
  }

  async fn report_statistics(&self, id: Option<String>) -> RpcResult<Vec<vino_rpc::Statistics>> {
    // TODO Dummy implementation
    if id.is_some() {
      Ok(vec![Statistics {
        num_calls: 1,
        execution_duration: ExecutionStatistics {
          max_time: 0,
          min_time: 0,
          average: 0,
        },
      }])
    } else {
      Ok(vec![Statistics {
        num_calls: 0,
        execution_duration: ExecutionStatistics {
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
  use log::debug;
  use maplit::hashmap;
  use vino_codec::messagepack::{
    deserialize,
    serialize,
  };
  use vino_component::{
    v0,
    Packet,
  };
  use vino_rpc::{
    Component,
    HostedType,
    Port,
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

    let mut outputs = provider
      .request(invocation_id.to_string(), "log".to_string(), job_payload)
      .await
      .expect("request failed");
    let (port_name, output) = outputs.next().await.unwrap();
    println!("Received payload from [{}]", port_name);
    let payload: String = match output {
      Packet::V0(v0::Payload::MessagePack(payload)) => deserialize(&payload)?,
      _ => None,
    }
    .unwrap();

    println!("outputs: {:?}", payload);
    assert_eq!(payload, "some_input");

    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn list() -> Result<()> {
    let provider = Provider::default();

    let response = provider.list_registered().await.expect("request failed");

    debug!("list response : {:?}", response);

    assert_eq!(response.len(), 4);

    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn statistics() -> Result<()> {
    let provider = Provider::default();

    let response = provider
      .report_statistics(None)
      .await
      .expect("request failed");

    debug!("statistics response : {:?}", response);

    assert_eq!(response.len(), 1);

    Ok(())
  }
}
