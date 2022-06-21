use futures::future::BoxFuture;
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::Instrument;
use wasmflow_interpreter::{BoxError, Collection};
use wasmflow_rpc::SharedRpcHandler;

use crate::dev::prelude::*;
type Result<T> = std::result::Result<T, CollectionError>;

pub(crate) struct NativeCollectionService {
  signature: CollectionSignature,
  collection: SharedRpcHandler,
}

impl NativeCollectionService {
  pub(crate) fn new(collection: SharedRpcHandler) -> Self {
    let HostedType::Collection(signature) = &collection.get_list().unwrap()[0];

    Self {
      collection,
      signature: signature.clone(),
    }
  }
}

impl Collection for NativeCollectionService {
  fn handle(
    &self,
    invocation: Invocation,
    _data: Option<serde_json::Value>,
  ) -> BoxFuture<std::result::Result<TransportStream, BoxError>> {
    let collection = self.collection.clone();

    async move {
      let mut receiver = collection.invoke(invocation).await?;
      let (tx, rx) = unbounded_channel();

      tokio::spawn(async move {
        while let Some(output) = receiver.next().await {
          if let Err(e) = tx.send(TransportWrapper {
            port: output.port,
            payload: output.payload,
          }) {
            error!("Error sending output on channel {}", e);
            break;
          }
        }
      });

      let rx = UnboundedReceiverStream::new(rx);

      Ok(TransportStream::new(rx))
    }
    .boxed()
  }

  fn list(&self) -> &CollectionSignature {
    &self.signature
  }

  fn shutdown(&self) -> BoxFuture<std::result::Result<(), BoxError>> {
    let collection = self.collection.clone();
    Box::pin(async move {
      collection.shutdown().await?;
      Ok(())
    })
  }
}

impl InvocationHandler for NativeCollectionService {
  fn get_signature(&self) -> Result<CollectionSignature> {
    let collection = self.collection.clone();

    let mut list = collection.get_list()?;
    drop(collection);

    match list.swap_remove(0) {
      HostedType::Collection(sig) => Ok(sig),
    }
  }

  fn invoke(&self, invocation: Invocation) -> Result<BoxFuture<Result<InvocationResponse>>> {
    let tx_id = invocation.tx_id;
    let span = debug_span!("invoke", target =  %invocation.target);
    let fut = self.handle(invocation, None);

    Ok(
      async move {
        Ok(crate::dispatch::InvocationResponse::Stream {
          tx_id,
          rx: fut.instrument(span).await?,
        })
      }
      .boxed(),
    )
  }
}

#[cfg(test)]
mod test {

  use std::sync::Arc;

  use anyhow::Result;
  use seeded_random::Seed;

  use super::*;
  use crate::test::prelude::assert_eq;

  #[test_logger::test(tokio::test)]
  async fn test_collection_component() -> Result<()> {
    let seed: u64 = 100000;
    let collection = NativeCollectionService::new(Arc::new(wasmflow_stdlib::Collection::new(Seed::unsafe_new(seed))));

    let user_data = "This is my payload";

    let payload = vec![("input", user_data)].into();
    let invocation = Invocation::new(Entity::test("test"), Entity::local("core::log"), payload, None);
    let response = collection.invoke(invocation)?.await?;

    let mut rx = response.ok()?;
    let next = rx.drain_port("output").await?[0].clone();
    let payload: String = next.payload.deserialize()?;
    assert_eq!(user_data, payload);

    Ok(())
  }
}
