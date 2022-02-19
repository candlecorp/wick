use futures::future::BoxFuture;
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;
use vino_rpc::SharedRpcHandler;

use crate::dev::prelude::*;
type Result<T> = std::result::Result<T, ProviderError>;

static PREFIX: &str = "NATIVE";

pub(crate) struct NativeProviderService {
  namespace: String,
  provider: SharedRpcHandler,
}

impl NativeProviderService {
  pub(crate) fn new(namespace: String, provider: SharedRpcHandler) -> Self {
    Self { namespace, provider }
  }
}

impl InvocationHandler for NativeProviderService {
  fn get_signature(&self) -> Result<ProviderSignature> {
    trace!("{}:InitComponents:[NS:{}]", PREFIX, self.namespace);

    let provider = self.provider.clone();

    let mut list = provider.get_list()?;
    drop(provider);

    match list.swap_remove(0) {
      HostedType::Provider(sig) => Ok(sig),
    }
  }

  fn invoke(&self, msg: InvocationMessage) -> Result<BoxFuture<Result<InvocationResponse>>> {
    trace!("{}:INVOKE:[{}]=>[{}]", PREFIX, msg.get_origin(), msg.get_target());

    let provider = self.provider.clone();

    let tx_id = msg.get_tx_id().to_owned();
    let component = msg.get_target().clone();
    let invocation = msg.into_inner();
    let url = component.url();

    Ok(
      async move {
        let receiver = provider.invoke(invocation).await;
        drop(provider);
        let (tx, rx) = unbounded_channel();
        match receiver {
          Ok(mut receiver) => {
            trace!("{}[{}]:START", PREFIX, url);
            tokio::spawn(async move {
              while let Some(output) = receiver.next().await {
                trace!("{}[{}]:PORT[{}]:RECV", PREFIX, url, output.port);

                if let Err(e) = tx.send(TransportWrapper {
                  port: output.port,
                  payload: output.payload,
                }) {
                  error!("Error sending output on channel {}", e.to_string());
                  break;
                }
              }
              trace!("{}[{}]:FINISH", PREFIX, url);
            });
          }
          Err(e) => {
            error!("Error invoking component: {}", e.to_string());
            let txresult = tx.send(TransportWrapper::component_error(MessageTransport::error(
              e.to_string(),
            )));
            let _ = map_err!(txresult, InternalError::E7002);
          }
        }

        let rx = UnboundedReceiverStream::new(rx);

        Ok(InvocationResponse::stream(tx_id, rx))
      }
      .boxed(),
    )
  }
}

#[cfg(test)]
mod test {

  use std::sync::Arc;

  use super::*;
  use crate::test::prelude::assert_eq;
  type Result<T> = super::Result<T>;

  #[test_logger::test(tokio::test)]
  async fn test_provider_component() -> Result<()> {
    let seed: u64 = 100000;
    let provider = NativeProviderService::new(
      "native-provider".to_owned(),
      Arc::new(vino_native_api_0::Provider::new(seed)),
    );

    let user_data = "This is my payload";

    let payload = vec![("input", user_data)].into();
    let invocation: InvocationMessage =
      Invocation::new(Entity::test("test"), Entity::component_direct("log"), payload).into();
    let response = provider.invoke(invocation)?.await?;

    let mut rx = response.ok()?;
    let next: TransportWrapper = rx.next().await.unwrap();
    let payload: String = next.payload.try_into()?;
    assert_eq!(user_data, payload);

    Ok(())
  }
}
