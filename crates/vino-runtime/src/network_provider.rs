use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};

use async_trait::async_trait;
use vino_provider::error::ProviderError;
use vino_rpc::port::PortStream;
use vino_rpc::{
  ExecutionStatistics,
  RpcHandler,
  RpcResult,
  Statistics,
};

use crate::network::Request;
use crate::Network;

#[derive(Debug, Default)]
pub struct State {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug, Default)]
pub struct Provider {
  network_id: String,
  context: Arc<Mutex<State>>,
}

impl Provider {
  pub fn new(network_id: String) -> Self {
    Self {
      network_id,
      context: Arc::new(Mutex::new(State::default())),
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
  ) -> RpcResult<PortStream> {
    let addr = Network::for_id(&self.network_id);
    let result = addr
      .send(Request {
        schematic: component,
        data: payload,
      })
      .await?;
    let context = self.context.clone();
    trace!("Provider running component {}", component);
    match components::get_component(&component) {
      Some(component) => {
        let future = component.job_wrapper(context, payload);
        let outputs = future.await?;
        Ok(outputs)
      }
      None => Err(ProviderError::ComponentNotFound(component.to_string()).into()),
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
  use log::trace;
  use maplit::hashmap;
  use vino_codec::messagepack::serialize;

  use super::*;
  use crate::Result;

  #[test]
  fn is_send() {
    let h = Provider::default();
    assert_is_send(h);
  }

  fn assert_is_send<T: Sync + Send>(_input: T) {}

  async fn add_item(
    provider: &Provider,
    document_id: &str,
    collection_id: &str,
    document: &str,
  ) -> Result<()> {
    let job_payload = hashmap! {
      "document_id".to_string() => serialize(document_id)?,
      "collection_id".to_string() => serialize(collection_id)?,
      "document".to_string() => serialize(document)?
    };
    let invocation_id = "INVOCATION_ID";

    let mut outputs = provider
      .request(
        invocation_id.to_string(),
        "add-item".to_string(),
        job_payload,
      )
      .await?;
    let (port, payload) = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", port, payload);
    let doc_id: String = payload.try_into()?;

    println!("doc_id: {:?}", doc_id);
    assert_eq!(doc_id, document_id);
    Ok(())
  }

  async fn get_item(provider: &Provider, document_id: &str, collection_id: &str) -> Result<String> {
    let job_payload = hashmap! {
      "document_id".to_string() => serialize(document_id)?,
      "collection_id".to_string() => serialize(collection_id)?,
    };
    let invocation_id = "INVOCATION_ID";

    let mut outputs = provider
      .request(
        invocation_id.to_string(),
        "get-item".to_string(),
        job_payload,
      )
      .await?;

    let (port, payload) = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", port, payload);
    let document: String = payload.try_into()?;

    println!("document: {:?}", document);
    Ok(document)
  }

  async fn list_items(provider: &Provider, collection_id: &str) -> Result<Vec<String>> {
    let job_payload = hashmap! {
      "collection_id".to_string() => serialize(collection_id)?,
    };
    let invocation_id = "INVOCATION_ID";

    let mut outputs = provider
      .request(
        invocation_id.to_string(),
        "list-items".to_string(),
        job_payload,
      )
      .await?;

    let (port, payload) = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", port, payload);
    let document_ids: Vec<String> = payload.try_into()?;

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
