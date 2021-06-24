use futures::prelude::*;
use log::debug;
use maplit::hashmap;
use test_vino_provider::Provider;
use vino_codec::messagepack::{
  deserialize,
  serialize,
};
use vino_component::{
  v0,
  Output,
};
use vino_rpc::RpcHandler;

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
    Output::V0(v0::Payload::MessagePack(payload)) => deserialize(&payload)?,
    _ => None,
  }
  .unwrap();

  println!("outputs: {:?}", payload);
  assert_eq!(payload, "some_input");

  Ok(())
}

#[test_env_log::test(tokio::test)]
async fn list() -> anyhow::Result<()> {
  let provider = Provider::default();

  let response = provider.list_registered().await.expect("request failed");
  debug!("list response : {:?}", response);

  Ok(())
}
