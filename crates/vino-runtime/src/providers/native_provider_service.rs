use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;
use vino_rpc::{
  clone_box,
  BoxedRpcHandler,
};

use crate::dev::prelude::*;
use crate::error::ProviderError;
type Result<T> = std::result::Result<T, ProviderError>;

static PREFIX: &str = "NATIVE";

#[derive(Default)]
pub(crate) struct NativeProviderService {
  namespace: String,
  state: Option<State>,
}

struct State {
  provider: BoxedRpcHandler,
}

impl Actor for NativeProviderService {
  type Context = Context<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    trace!("{}:Service:Start", PREFIX);
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {}
}

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
  pub(crate) namespace: String,
  pub(crate) provider: BoxedRpcHandler,
}

impl Handler<Initialize> for NativeProviderService {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {
    self.namespace = msg.namespace;
    trace!("{}:Init:{}", PREFIX, self.namespace);

    self.state = Some(State {
      provider: msg.provider,
    });
    ActorResult::reply(Ok(()))
  }
}

#[derive(Message)]
#[rtype(result = "Result<ProviderSignature>")]
pub(crate) struct InitializeComponents {}

impl Handler<InitializeComponents> for NativeProviderService {
  type Result = ActorResult<Self, Result<ProviderSignature>>;

  fn handle(&mut self, _msg: InitializeComponents, _ctx: &mut Self::Context) -> Self::Result {
    trace!("{}:InitComponents:[NS:{}]", PREFIX, self.namespace);

    let state = some_or_bail!(
      &self.state,
      ActorResult::reply(Err(ProviderError::Uninitialized))
    );
    let provider = clone_box(&*state.provider);

    let task = async move {
      let mut list = provider.get_list().await?;
      drop(provider);

      match list.swap_remove(0) {
        HostedType::Provider(sig) => Ok(sig),
      }
    };

    ActorResult::reply_async(task.into_actor(self))
  }
}

impl Handler<InvocationMessage> for NativeProviderService {
  type Result = ActorResult<Self, InvocationResponse>;

  fn handle(&mut self, msg: InvocationMessage, _ctx: &mut Self::Context) -> Self::Result {
    trace!(
      "{}:INVOKE:[{}]=>[{}]",
      PREFIX,
      msg.get_origin(),
      msg.get_target()
    );

    let state = self.state.as_ref().unwrap();
    let provider = clone_box(&*state.provider);

    let tx_id = msg.get_tx_id().to_owned();
    let component = msg.get_target().clone();
    let message = msg.get_payload_owned();
    let url = component.url();

    let request = async move {
      let receiver = provider.invoke(component, message).await;
      drop(provider);
      let (tx, rx) = unbounded_channel();
      match receiver {
        Ok(mut receiver) => {
          trace!("{}[{}]:START", PREFIX, url);
          actix::spawn(async move {
            loop {
              trace!("{}[{}]:WAIT", PREFIX, url);
              let output = match receiver.next().await {
                Some(v) => v,
                None => break,
              };
              trace!("{}[{}]:PORT[{}]:RECV", PREFIX, url, output.port);
              match tx.send(TransportWrapper {
                port: output.port.clone(),
                payload: output.payload,
              }) {
                Ok(_) => {
                  trace!("{}[{}]:PORT[{}]:SENT", PREFIX, url, output.port);
                }
                Err(e) => {
                  error!("Error sending output on channel {}", e.to_string());
                  break;
                }
              }
            }
          });
        }
        Err(e) => {
          error!("Error invoking component: {}", e.to_string());
          let txresult = tx.send(TransportWrapper {
            port: vino_transport::COMPONENT_ERROR.to_owned(),
            payload: MessageTransport::error(e.to_string()),
          });
          let _ = map_err!(txresult, InternalError::E7002);
        }
      }

      let rx = UnboundedReceiverStream::new(rx);

      InvocationResponse::stream(tx_id, rx)
    };
    ActorResult::reply_async(request.into_actor(self))
  }
}

#[cfg(test)]
mod test {

  use super::*;
  use crate::test::prelude::assert_eq;
  type Result<T> = super::Result<T>;

  #[test_logger::test(actix_rt::test)]
  async fn test_provider_component() -> Result<()> {
    let provider = NativeProviderService::default();
    let addr = provider.start();
    let seed: u64 = 100000;
    addr
      .send(Initialize {
        namespace: "native-provider".to_owned(),
        provider: Box::new(vino_native_api_0::Provider::new(seed)),
      })
      .await??;

    let user_data = "This is my payload";

    let payload = vec![("input", user_data)].into();
    let invocation: InvocationMessage = Invocation::new(
      Entity::test("test"),
      Entity::component_direct("log"),
      payload,
    )
    .into();
    let response = addr.send(invocation).await?;

    let mut rx = response.ok()?;
    let next: TransportWrapper = rx.next().await.unwrap();
    let payload: String = next.payload.try_into()?;
    assert_eq!(user_data, payload);

    Ok(())
  }
}
