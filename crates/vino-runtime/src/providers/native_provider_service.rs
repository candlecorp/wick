use std::collections::HashMap;

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

#[derive(Debug, Default)]
pub(crate) struct NativeProviderService {
  namespace: String,
  state: Option<State>,
}

#[derive(Derivative)]
#[derivative(Debug)]
struct State {
  #[derivative(Debug = "ignore")]
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
    self.namespace = msg.namespace.clone();
    trace!("{}:Init:{}", PREFIX, self.namespace);

    self.state = Some(State {
      provider: msg.provider,
    });
    ActorResult::reply(Ok(()))
  }
}

#[derive(Message)]
#[rtype(result = "Result<HashMap<String, ComponentModel>>")]
pub(crate) struct InitializeComponents {}

impl Handler<InitializeComponents> for NativeProviderService {
  type Result = ActorResult<Self, Result<HashMap<String, ComponentModel>>>;

  fn handle(&mut self, _msg: InitializeComponents, _ctx: &mut Self::Context) -> Self::Result {
    trace!("{}:InitComponents:[NS:{}]", PREFIX, self.namespace);

    let state = some_or_bail!(
      &self.state,
      ActorResult::reply(Err(ProviderError::Uninitialized))
    );
    let provider = clone_box(&*state.provider);
    let namespace = self.namespace.clone();

    let task = async move {
      let list = provider.get_list().await?;
      drop(provider);

      let mut metadata: HashMap<String, ComponentModel> = HashMap::new();

      for item in list {
        match item {
          HostedType::Component(component) => {
            metadata.insert(
              component.name.clone(),
              ComponentModel {
                namespace: namespace.clone(),
                name: component.name,
                inputs: component.inputs.into_iter().map(From::from).collect(),
                outputs: component.outputs.into_iter().map(From::from).collect(),
              },
            );
          }
          HostedType::Schematic(component) => {
            metadata.insert(
              component.name.clone(),
              ComponentModel {
                namespace: namespace.clone(),
                name: component.name,
                inputs: component.inputs.into_iter().map(From::from).collect(),
                outputs: component.outputs.into_iter().map(From::from).collect(),
              },
            );
          }
        }
      }
      Ok(metadata)
    };

    ActorResult::reply_async(task.into_actor(self))
  }
}

impl Handler<Invocation> for NativeProviderService {
  type Result = ActorResult<Self, InvocationResponse>;

  fn handle(&mut self, msg: Invocation, _ctx: &mut Self::Context) -> Self::Result {
    trace!(
      "{}:[NS:{}]:Invoke: {} to {}",
      PREFIX,
      self.namespace,
      msg.origin.url(),
      msg.target.url()
    );
    let ns = self.namespace.clone();

    let state = self.state.as_ref().unwrap();
    let provider = clone_box(&*state.provider);

    let tx_id = msg.tx_id.clone();
    let component = msg.target;
    let message = msg.msg;
    let url = component.url();

    let request = async move {
      let receiver = provider.invoke(component, message).await;
      drop(provider);
      if let Err(e) = receiver {
        return InvocationResponse::error(
          tx_id,
          format!("Provider component {} failed: {}", url, e.to_string()),
        );
      }
      let mut receiver = receiver.unwrap();
      let (tx, rx) = unbounded_channel();
      actix::spawn(async move {
        loop {
          trace!("{}:[NS:{}]:{}:WAIT", PREFIX, ns, url);
          let output = match receiver.next().await {
            Some(v) => v,
            None => break,
          };
          trace!("{}:[NS:{}]:{}:PORT:{}:RECV", PREFIX, ns, url, output.port);
          match tx.send(TransportWrapper {
            port: output.port.clone(),
            payload: output.payload,
          }) {
            Ok(_) => {
              trace!("{}:[NS:{}]:{}:PORT:{}:SENT", PREFIX, ns, url, output.port);
            }
            Err(e) => {
              error!("Error sending output on channel {}", e.to_string());
              break;
            }
          }
        }
      });
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
    addr
      .send(Initialize {
        namespace: "native-provider".to_owned(),
        provider: Box::new(vino_native_api_0::Provider::default()),
      })
      .await??;

    let user_data = "This is my payload";

    let payload = transport_map! {"input" => user_data};

    let response = addr
      .send(Invocation {
        origin: Entity::test("test"),
        target: Entity::component_direct("log"),
        msg: payload,
        id: get_uuid(),
        tx_id: get_uuid(),
      })
      .await?;

    let mut rx = response.ok()?;
    let next: TransportWrapper = rx.next().await.unwrap();
    let payload: String = next.payload.try_into()?;
    assert_eq!(user_data, payload);

    Ok(())
  }
}
