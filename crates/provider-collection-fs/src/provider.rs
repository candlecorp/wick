use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{
  Arc,
  Mutex,
};

use async_trait::async_trait;
use vino_provider::entity::Entity;
use vino_provider::error::ProviderError;
use vino_rpc::{
  BoxedPacketStream,
  DurationStatistics,
  RpcHandler,
  RpcResult,
};

use crate::generated;

#[derive(Debug)]
pub struct State {
  pub directory: PathBuf,
}

#[derive(Clone, Debug)]
#[must_use]
pub struct Provider {
  context: Arc<Mutex<State>>,
}

impl Provider {
  pub fn new(dir: PathBuf) -> Self {
    Self {
      context: Arc::new(Mutex::new(State { directory: dir })),
    }
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(
    &self,
    entity: Entity,
    payload: HashMap<String, Vec<u8>>,
  ) -> RpcResult<BoxedPacketStream> {
    let context = self.context.clone();
    let component = entity.into_component()?;
    trace!("Provider running component {}", component);
    match generated::get_component(&component) {
      Some(component) => {
        let future = component.job_wrapper(context, payload);
        let outputs = future.await?;
        Ok(Box::pin(outputs))
      }
      None => Err(ProviderError::ComponentNotFound(component).into()),
    }
  }

  async fn get_list(&self) -> RpcResult<Vec<vino_rpc::HostedType>> {
    let components = generated::get_all_components();
    Ok(
      components
        .into_iter()
        .map(vino_rpc::HostedType::Component)
        .collect(),
    )
  }

  async fn get_stats(&self, id: Option<String>) -> RpcResult<Vec<vino_rpc::Statistics>> {
    // TODO Dummy implementation
    if id.is_some() {
      Ok(vec![vino_rpc::Statistics {
        num_calls: 1,
        execution_duration: DurationStatistics {
          max_time: 0,
          min_time: 0,
          average: 0,
        },
      }])
    } else {
      Ok(vec![vino_rpc::Statistics {
        num_calls: 0,
        execution_duration: DurationStatistics {
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
  use anyhow::Result;
  use futures::prelude::*;
  use vino_component::{
    v0,
    Packet,
  };
  use vino_rpc::make_input;

  use super::*;

  async fn add_item(
    provider: &Provider,
    document_id: &str,
    collection_id: &str,
    document: &str,
  ) -> Result<()> {
    let job_payload = make_input(vec![
      ("document_id", document_id),
      ("collection_id", collection_id),
      ("document", document),
    ]);

    let mut outputs = provider
      .invoke(Entity::component("add-item"), job_payload)
      .await?;
    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.packet);
    let doc_id: String = output.packet.try_into()?;

    println!("doc_id: {:?}", doc_id);
    assert_eq!(doc_id, document_id);
    Ok(())
  }

  async fn get_item(provider: &Provider, document_id: &str, collection_id: &str) -> Result<String> {
    let job_payload = make_input(vec![
      ("document_id", document_id),
      ("collection_id", collection_id),
    ]);

    let mut outputs = provider
      .invoke(Entity::component("get-item"), job_payload)
      .await?;

    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.packet);
    let document: String = output.packet.try_into()?;

    println!("document: {:?}", document);
    Ok(document)
  }

  async fn rm_item(provider: &Provider, document_id: &str, collection_id: &str) -> Result<()> {
    let job_payload = make_input(vec![
      ("document_id", document_id),
      ("collection_id", collection_id),
    ]);

    let mut outputs = provider
      .invoke(Entity::component("rm-item"), job_payload.clone())
      .await?;
    let output = outputs.next().await;
    assert!(output.is_none());

    let mut outputs = provider
      .invoke(Entity::component("get-item"), job_payload)
      .await?;

    let output = outputs.next().await.unwrap();
    println!("Output of post-rm get: {:?}", output);
    assert!(matches!(
      output.packet,
      Packet::V0(v0::Payload::Exception(_))
    ));
    Ok(())
  }

  async fn list_items(provider: &Provider, collection_id: &str) -> Result<Vec<String>> {
    let job_payload = make_input(vec![("collection_id", collection_id)]);

    let mut outputs = provider
      .invoke(Entity::component("list-items"), job_payload)
      .await?;

    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.packet);
    let document_ids: Vec<String> = output.packet.try_into()?;

    println!("document_ids: {:?}", document_ids);
    Ok(document_ids)
  }

  #[test_env_log::test(tokio::test)]
  async fn request_add_item() -> Result<()> {
    let provider = Provider::new(std::env::temp_dir());
    let document_id = "some_doc_id1";
    let collection_id = "some_collection_id1";
    let document = "This is my document";
    add_item(&provider, document_id, collection_id, document).await?;
    rm_item(&provider, document_id, collection_id).await?;
    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn request_get_item() -> Result<()> {
    let provider = Provider::new(std::env::temp_dir());
    let document_id = "some_doc_id2";
    let collection_id = "some_collection_id2";
    let document = "This is my document";
    add_item(&provider, document_id, collection_id, document).await?;
    let doc = get_item(&provider, document_id, collection_id).await?;
    trace!("Doc is {}", doc);
    assert_eq!(doc, document);
    rm_item(&provider, document_id, collection_id).await?;

    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn request_list_items() -> Result<()> {
    let provider = Provider::new(std::env::temp_dir());
    let collection_id = "some_collection_id3";
    let doc_id1 = "doc_id_1";
    let doc_id2 = "doc_id_2";

    add_item(&provider, doc_id1, collection_id, "content 1").await?;
    add_item(&provider, doc_id2, collection_id, "content 2").await?;
    let docs = list_items(&provider, collection_id).await?;
    assert_eq!(docs.len(), 2);
    rm_item(&provider, doc_id1, collection_id).await?;
    rm_item(&provider, doc_id2, collection_id).await?;
    let docs = list_items(&provider, collection_id).await?;
    assert_eq!(docs.len(), 0);

    Ok(())
  }
}
