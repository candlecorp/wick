use anyhow::Result;
use vino_invocation_server::{
  bind_new_socket,
  make_rpc_server,
};
use vino_rpc::rpc::invocation_service_client::InvocationServiceClient;
use vino_rpc::rpc::ListRequest;
use vino_rpc::SharedRpcHandler;

async fn list_components(port: &u16) -> Result<Vec<vino_rpc::rpc::HostedType>> {
  let mut client = InvocationServiceClient::connect(format!("http://127.0.0.1:{}", port)).await?;
  let request = ListRequest {};
  let response = client.list(request).await?.into_inner();

  println!("Output = {:?}", response);
  Ok(response.schemas)
}

pub async fn test_api(provider: SharedRpcHandler) -> Result<()> {
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
