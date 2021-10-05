use std::collections::HashMap;

use anyhow::Result;
use log::*;
use maplit::hashmap;
use vino_codec::messagepack::serialize;
use vino_invocation_server::{
  bind_new_socket,
  make_rpc_server,
  InvocationClient,
};
use vino_packet::Packet;
use vino_provider::native::prelude::*;
use vino_rpc::rpc::invocation_service_client::InvocationServiceClient;
use vino_rpc::rpc::{
  Invocation,
  ListRequest,
};
use vino_rpc::{
  convert_transport_map,
  BoxedRpcHandler,
};

async fn list_components(port: &u16) -> Result<Vec<vino_rpc::rpc::HostedType>> {
  let mut client = InvocationServiceClient::connect(format!("http://127.0.0.1:{}", port)).await?;
  let request = ListRequest {};
  let response = client.list(request).await?.into_inner();

  println!("Output = {:?}", response);
  Ok(response.schemas)
}

fn make_invocation(origin: &str, target: &str, payload: TransportMap) -> Result<Invocation> {
  Ok(Invocation {
    origin: Entity::test(origin).url(),
    target: Entity::component_direct(target).url(),
    msg: convert_transport_map(payload),
    id: "".to_string(),
  })
}

pub async fn test_api(provider: BoxedRpcHandler) -> Result<()> {
  let socket = bind_new_socket()?;
  let port = socket.local_addr()?.port();
  let _server = make_rpc_server(socket, provider);

  let components = list_components(&port).await?;
  println!("Reported components: {:#?}", components);
  assert_eq!(components.len(), 4);
  // request_add_item(&port).await?;
  // request_get_item(&port).await?;
  // request_list_items(&port).await?;
  // request_rm_item(&port).await?;
  Ok(())
}
