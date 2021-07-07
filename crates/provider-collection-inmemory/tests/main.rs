use std::collections::HashMap;

use maplit::hashmap;
use tonic::transport::Channel;
use vino_codec::messagepack::serialize;
use vino_collection_inmemory::error::InMemoryCollectionError;
use vino_collection_inmemory::Result;
use vino_component::Packet;
use vino_provider::entity::Entity;
use vino_rpc::rpc::invocation_service_client::InvocationServiceClient;
use vino_rpc::rpc::Invocation;

#[macro_use]
extern crate tracing;

fn from_tonic(e: tonic::Status) -> InMemoryCollectionError {
  InMemoryCollectionError::Other(e.to_string())
}

fn make_invocation(
  origin: &str,
  target: &str,
  payload: HashMap<String, Vec<u8>>,
) -> Result<Invocation> {
  Ok(Invocation {
    origin: Entity::test(origin).url(),
    target: Entity::component(target).url(),
    msg: payload,
    id: "".to_string(),
    tx_id: "".to_string(),
    encoded_claims: "".to_string(),
    network_id: "".to_string(),
  })
}

async fn add_item(
  client: &mut InvocationServiceClient<Channel>,
  coll_id: &str,
  doc_id: &str,
  doc: &str,
) -> Result<String> {
  let payload = hashmap! {
    "document_id".to_string()=> serialize(doc_id.to_string())?,
    "document".to_string()=> serialize(doc.to_string())?,
    "collection_id".to_string()=> serialize(coll_id.to_string())?,
  };
  let invocation = make_invocation("add_item", "add-item", payload)?;
  let invocation_id = invocation.id.to_string();
  let mut stream = client
    .invoke(invocation)
    .await
    .map_err(from_tonic)?
    .into_inner();

  let next = stream.message().await.map_err(from_tonic)?.unwrap();
  println!("Output = {:?}", next);
  assert_eq!(next.invocation_id, invocation_id);
  assert_eq!(next.port, "document_id");
  let next: Packet = next.payload.unwrap().into();
  Ok(next.try_into()?)
}

async fn get_item(
  client: &mut InvocationServiceClient<Channel>,
  collection_id: &str,
  document_id: &str,
) -> Result<String> {
  let payload = hashmap! {
    "document_id".to_string() => serialize(document_id)?,
    "collection_id".to_string() => serialize(collection_id)?,
  };

  let invocation = make_invocation("add-item", "get-item", payload)?;
  let invocation_id = invocation.id.to_string();
  let mut stream = client
    .invoke(invocation)
    .await
    .map_err(from_tonic)?
    .into_inner();

  let next = stream.message().await.map_err(from_tonic)?.unwrap();
  println!("Output = {:?}", next);
  assert_eq!(next.invocation_id, invocation_id);
  assert_eq!(next.port, "document");
  let next: Packet = next.payload.unwrap().into();
  Ok(next.try_into()?)
}

async fn list_items(
  client: &mut InvocationServiceClient<Channel>,
  collection_id: &str,
) -> Result<Vec<String>> {
  let payload = hashmap! {
    "collection_id".to_string() => serialize(collection_id)?,
  };

  let invocation = make_invocation("list-item", "list-items", payload)?;
  let invocation_id = invocation.id.to_string();
  let mut stream = client
    .invoke(invocation)
    .await
    .map_err(from_tonic)?
    .into_inner();

  let next = stream.message().await.map_err(from_tonic)?.unwrap();
  println!("Output = {:?}", next);
  assert_eq!(next.invocation_id, invocation_id);
  assert_eq!(next.port, "document_ids");
  let next: Packet = next.payload.unwrap().into();
  Ok(next.try_into()?)
}

#[test_env_log::test(tokio::test)]
async fn request_add_item() -> Result<()> {
  let socket = vino_rpc::bind_new_socket()?;
  let port = socket.local_addr()?.port();
  vino_rpc::make_rpc_server(
    socket,
    vino_collection_inmemory::provider::Provider::default(),
  );

  let mut client = InvocationServiceClient::connect(format!("http://127.0.0.1:{}", port))
    .await
    .map_err(|_e| InMemoryCollectionError::Other("connect error".into()))?;
  info!("Connected to server");
  let document_id = "some_doc_id";
  let collection_id = "some_collection_id";
  let document = "This is my document";
  add_item(&mut client, collection_id, document_id, document).await?;
  Ok(())
}

#[test_env_log::test(tokio::test)]
async fn request_get_item() -> Result<()> {
  let socket = vino_rpc::bind_new_socket()?;
  let port = socket.local_addr()?.port();
  vino_rpc::make_rpc_server(
    socket,
    vino_collection_inmemory::provider::Provider::default(),
  );

  let mut client = InvocationServiceClient::connect(format!("http://127.0.0.1:{}", port))
    .await
    .map_err(|_e| InMemoryCollectionError::Other("connect error".into()))?;
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

#[test_env_log::test(tokio::test)]
async fn request_list_items() -> Result<()> {
  let socket = vino_rpc::bind_new_socket()?;
  let port = socket.local_addr()?.port();
  vino_rpc::make_rpc_server(
    socket,
    vino_collection_inmemory::provider::Provider::default(),
  );

  let mut client = InvocationServiceClient::connect(format!("http://127.0.0.1:{}", port))
    .await
    .map_err(|_e| InMemoryCollectionError::Other("connect error".into()))?;
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
