use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};

use vino_provider::native::prelude::*;
use vino_rpc::error::RpcError;
use vino_rpc::{
  RpcHandler,
  RpcResult,
};

use crate::generated::{
  self,
  Dispatcher,
};

pub(crate) type Context = Arc<Mutex<State>>;

#[derive(Debug, Default)]
pub struct State {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug, Default)]
#[must_use]
pub struct Provider {
  context: Arc<Mutex<State>>,
}

impl Provider {
  pub fn default() -> Self {
    Self {
      context: Arc::new(Mutex::new(State::default())),
    }
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, entity: Entity, payload: TransportMap) -> RpcResult<BoxedTransportStream> {
    let context = self.context.clone();
    debug!("Dispatching to {}", entity.url());
    let component = entity.name();
    let stream = Dispatcher::dispatch(&component, context, payload)
      .await
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;

    Ok(Box::pin(stream))
  }

  async fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let components = generated::get_all_components();
    Ok(components.into_iter().map(HostedType::Component).collect())
  }
}

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use futures::prelude::*;
  use vino_macros::transport_map;

  use super::*;

  async fn add_item(
    provider: &Provider,
    document_id: &str,
    collection_id: &str,
    document: &str,
  ) -> Result<()> {
    let job_payload = transport_map! {
      "document_id"=> document_id,
      "collection_id"=> collection_id,
      "document"=> document,
    };

    let mut outputs = provider
      .invoke(Entity::component_direct("add-item"), job_payload)
      .await?;
    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let doc_id: String = output.payload.try_into()?;

    println!("doc_id: {:?}", doc_id);
    assert_eq!(doc_id, document_id);
    Ok(())
  }

  async fn get_item(provider: &Provider, document_id: &str, collection_id: &str) -> Result<String> {
    let job_payload = transport_map! {
      "document_id"=> document_id,
      "collection_id"=> collection_id,
    };
    let mut outputs = provider
      .invoke(Entity::component_direct("get-item"), job_payload)
      .await?;

    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let document: String = output.payload.try_into()?;

    println!("document: {:?}", document);
    Ok(document)
  }

  async fn list_items(provider: &Provider, collection_id: &str) -> Result<Vec<String>> {
    let job_payload = transport_map! {
      "collection_id"=> collection_id,
    };
    let mut outputs = provider
      .invoke(Entity::component_direct("list-items"), job_payload)
      .await?;

    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let document_ids: Vec<String> = output.payload.try_into()?;

    println!("document_ids: {:?}", document_ids);
    Ok(document_ids)
  }

  #[test_env_log::test(tokio::test)]
  async fn request_add_item() -> Result<()> {
    let provider = Provider::default();
    let document_id = "some_doc_id";
    let collection_id = "some_collection_id";
    let document = "This is my document";
    add_item(&provider, document_id, collection_id, document).await?;
    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn request_get_item() -> Result<()> {
    let provider = Provider::default();
    let document_id = "some_doc_id";
    let collection_id = "some_collection_id";
    let document = "This is my document";
    add_item(&provider, document_id, collection_id, document).await?;
    let doc = get_item(&provider, document_id, collection_id).await?;
    trace!("Doc is {}", doc);
    assert_eq!(doc, document);

    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn request_list_items() -> Result<()> {
    let provider = Provider::default();
    let collection_id = "some_collection_id";
    add_item(&provider, "doc_id_1", collection_id, "content 1").await?;
    add_item(&provider, "doc_id_2", collection_id, "content 2").await?;
    let docs = list_items(&provider, collection_id).await?;

    assert_eq!(docs.len(), 2);

    Ok(())
  }
}
