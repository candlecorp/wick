use futures::future::BoxFuture;
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::Instrument;
use vino_interpreter::{BoxError, Provider};
use vino_rpc::SharedRpcHandler;

use crate::dev::prelude::*;
type Result<T> = std::result::Result<T, ProviderError>;

pub(crate) struct NativeProviderService {
  signature: ProviderSignature,
  provider: SharedRpcHandler,
}

impl NativeProviderService {
  pub(crate) fn new(provider: SharedRpcHandler) -> Self {
    let HostedType::Provider(signature) = &provider.get_list().unwrap()[0];

    Self {
      provider,
      signature: signature.clone(),
    }
  }
}

impl Provider for NativeProviderService {
  fn handle(
    &self,
    invocation: Invocation,
    _data: Option<serde_json::Value>,
  ) -> BoxFuture<std::result::Result<TransportStream, BoxError>> {
    let provider = self.provider.clone();

    async move {
      let mut receiver = provider.invoke(invocation).await?;
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

  fn list(&self) -> &ProviderSignature {
    &self.signature
  }

  fn shutdown(&self) -> BoxFuture<std::result::Result<(), BoxError>> {
    let provider = self.provider.clone();
    Box::pin(async move {
      provider.shutdown().await?;
      Ok(())
    })
  }
}

impl InvocationHandler for NativeProviderService {
  fn get_signature(&self) -> Result<ProviderSignature> {
    let provider = self.provider.clone();

    let mut list = provider.get_list()?;
    drop(provider);

    match list.swap_remove(0) {
      HostedType::Provider(sig) => Ok(sig),
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

  use vino_random::Seed;

  use super::*;
  use crate::test::prelude::assert_eq;
  type Result<T> = super::Result<T>;

  #[test_logger::test(tokio::test)]
  async fn test_provider_component() -> Result<()> {
    let seed: u64 = 100000;
    let provider = NativeProviderService::new(Arc::new(vino_stdlib::Provider::new(Seed::unsafe_new(seed))));

    let user_data = "This is my payload";

    let payload = vec![("input", user_data)].into();
    let invocation = Invocation::new(
      Entity::test("test"),
      Entity::local_component("core::log"),
      payload,
      None,
    );
    let response = provider.invoke(invocation)?.await?;

    let mut rx = response.ok()?;
    let next: TransportWrapper = rx.next().await.unwrap();
    let payload: String = next.payload.deserialize()?;
    assert_eq!(user_data, payload);

    Ok(())
  }
}
