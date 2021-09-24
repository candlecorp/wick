use anyhow::Result;
use log::*;
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

async fn list_components(port: &u16) -> Result<Vec<vino_rpc::rpc::Component>> {
  let mut client = InvocationServiceClient::connect(format!("http://127.0.0.1:{}", port)).await?;
  let request = ListRequest {};
  let response = client.list(request).await?.into_inner();

  println!("Output = {:?}", response);
  Ok(response.components)
}

fn make_invocation(origin: &str, target: &str, payload: TransportMap) -> Result<Invocation> {
  Ok(Invocation {
    origin: Entity::test(origin).url(),
    target: Entity::component_direct(target).url(),
    msg: convert_transport_map(payload),
    id: "".to_string(),
  })
}

async fn add_item(
  client: &mut InvocationClient,
  collection_id: &str,
  document_id: &str,
  document: &str,
) -> Result<String> {
  let inputs = vino_interface_collection::generated::add_item::Inputs {
    document_id: document_id.to_owned(),
    document: document.to_owned(),
    collection_id: collection_id.to_owned(),
  };

  let invocation = make_invocation("add_item", "add-item", inputs.into())?;
  let invocation_id = invocation.id.to_string();
  let mut stream = client.invoke(invocation).await?.into_inner();

  let next = stream.message().await?.unwrap();
  println!("Output = {:?}", next);
  assert_eq!(next.invocation_id, invocation_id);
  assert_eq!(next.port, "document_id");
  let next: Packet = next.payload.unwrap().into();
  Ok(next.try_into()?)
}

async fn get_item(
  client: &mut InvocationClient,
  collection_id: &str,
  document_id: &str,
) -> Result<String> {
  let inputs = vino_interface_collection::generated::get_item::Inputs {
    document_id: document_id.to_owned(),
    collection_id: collection_id.to_owned(),
  };
  let invocation = make_invocation("get-item", "get-item", inputs.into())?;
  let invocation_id = invocation.id.to_string();
  let mut stream = client.invoke(invocation).await?.into_inner();

  let next = stream.message().await?.unwrap();
  println!("Output = {:?}", next);
  assert_eq!(next.invocation_id, invocation_id);
  assert_eq!(next.port, "document");
  let next: Packet = next.payload.unwrap().into();
  Ok(next.try_into()?)
}

async fn rm_item(
  client: &mut InvocationClient,
  collection_id: &str,
  document_id: &str,
) -> Result<()> {
  let inputs = vino_interface_collection::generated::rm_item::Inputs {
    document_id: document_id.to_owned(),
    collection_id: collection_id.to_owned(),
  };
  let invocation = make_invocation("rm-item", "rm-item", inputs.into())?;
  let mut stream = client.invoke(invocation).await?.into_inner();

  let next = stream.message().await?;
  assert!(next.is_none());
  Ok(())
}

async fn list_items(client: &mut InvocationClient, collection_id: &str) -> Result<Vec<String>> {
  let inputs = vino_interface_collection::generated::list_items::Inputs {
    collection_id: collection_id.to_owned(),
  };
  let invocation = make_invocation("list-item", "list-items", inputs.into())?;
  let invocation_id = invocation.id.to_string();
  let mut stream = client.invoke(invocation).await?.into_inner();

  let next = stream.message().await?.unwrap();
  println!("Output = {:?}", next);
  assert_eq!(next.invocation_id, invocation_id);
  assert_eq!(next.port, "document_ids");
  let next: Packet = next.payload.unwrap().into();
  Ok(next.try_into()?)
}

async fn request_add_item(port: &u16) -> Result<()> {
  let mut client = InvocationServiceClient::connect(format!("http://127.0.0.1:{}", port)).await?;
  info!("Connected to server");
  let document_id = "some_doc_id";
  let collection_id = "some_collection_id";
  let document = "This is my document";
  add_item(&mut client, collection_id, document_id, document).await?;
  Ok(())
}

async fn request_get_item(port: &u16) -> Result<()> {
  let mut client = InvocationServiceClient::connect(format!("http://127.0.0.1:{}", port)).await?;
  info!("Connected to server");
  let document_id = "some_doc_id";
  let collection_id = "some_collection_id";
  let document = "This is my document";
  add_item(&mut client, collection_id, document_id, document).await?;
  let doc = get_item(&mut client, collection_id, document_id).await?;
  trace!("Doc is {}", doc);
  assert_eq!(doc, document);

  Ok(())
}

async fn request_rm_item(port: &u16) -> Result<()> {
  let mut client = InvocationServiceClient::connect(format!("http://127.0.0.1:{}", port)).await?;
  info!("Connected to server");
  let document_id = "some_doc_id";
  let collection_id = "some_collection_id";
  let document = "This is my document";
  add_item(&mut client, collection_id, document_id, document).await?;
  rm_item(&mut client, collection_id, document_id).await?;
  let doc = get_item(&mut client, collection_id, document_id).await;
  trace!("Doc is {:?}", doc);
  assert!(doc.is_err());

  Ok(())
}

async fn request_list_items(port: &u16) -> Result<()> {
  let mut client = InvocationServiceClient::connect(format!("http://127.0.0.1:{}", port)).await?;
  let collection_id = "some_collection_id";
  add_item(&mut client, collection_id, "doc_id_1", "content 1").await?;
  add_item(&mut client, collection_id, "doc_id_2", "content 2").await?;
  let docs = list_items(&mut client, collection_id).await?;
  let mut found1 = false;
  let mut found2 = false;
  for doc_id in docs {
    trace!("doc id {}", doc_id);
    if doc_id.ends_with("doc_id_1") {
      found1 = true;
      let doc = get_item(&mut client, collection_id, &doc_id).await?;
      assert_eq!(doc, "content 1");
    } else if doc_id.ends_with("doc_id_2") {
      found2 = true;
      let doc = get_item(&mut client, collection_id, &doc_id).await?;
      assert_eq!(doc, "content 2");
    }
  }
  assert!(found1);
  assert!(found2);

  Ok(())
}

pub async fn test_api(provider: BoxedRpcHandler) -> Result<()> {
  let socket = bind_new_socket()?;
  let port = socket.local_addr()?.port();
  let _server = make_rpc_server(socket, provider);

  let components = list_components(&port).await?;
  println!("Reported components: {:#?}", components);
  assert_eq!(components.len(), 4);
  request_add_item(&port).await?;
  request_get_item(&port).await?;
  request_list_items(&port).await?;
  request_rm_item(&port).await?;
  Ok(())
}
