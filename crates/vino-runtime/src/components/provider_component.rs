use std::sync::Arc;

use actix::prelude::*;
use futures::StreamExt;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex;
use vino_rpc::RpcHandler;

use crate::actix::ActorResult;
use crate::schematic::NativeOutputReady;
use crate::{
  Invocation,
  InvocationResponse,
  Result,
};

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct ProviderComponent {
  name: String,
  seed: String,
  #[derivative(Debug = "ignore")]
  state: Option<State>,
}

struct State {
  provider: Arc<Mutex<dyn RpcHandler>>,
}

impl Actor for ProviderComponent {
  type Context = Context<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    trace!("Provider component started");
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {}
}

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
  pub(crate) name: String,
  pub(crate) seed: String,
  pub(crate) provider: Arc<Mutex<dyn RpcHandler>>,
}

impl Handler<Initialize> for ProviderComponent {
  type Result = Result<()>;

  fn handle(&mut self, msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {
    trace!("ProviderComponent initialized for {}", msg.name);
    self.name = msg.name;
    self.seed = msg.seed;
    self.state = Some(State {
      provider: msg.provider,
    });
    Ok(())
  }
}

impl Handler<Invocation> for ProviderComponent {
  type Result = ActorResult<Self, InvocationResponse>;

  fn handle(&mut self, msg: Invocation, _ctx: &mut Self::Context) -> Self::Result {
    let provider = self.state.as_ref().unwrap().provider.clone();
    let tx_id = msg.tx_id.clone();
    let component = actix_ensure_ok!(msg
      .target
      .into_component()
      .map_err(|_e| InvocationResponse::error(tx_id.clone(), "Sent invalid entity".to_string())));
    let message = actix_ensure_ok!(msg
      .msg
      .into_multibytes()
      .map_err(|_e| InvocationResponse::error(tx_id.clone(), "Sent invalid payload".to_string())));
    let name = component.name;
    let inv_id = msg.id;

    let request = async move {
      let provider = provider.lock().await;
      let invocation_id = inv_id.clone();
      let receiver = provider
        .request(inv_id.clone(), name.clone(), message)
        .await;
      if let Err(e) = receiver {
        return InvocationResponse::error(
          tx_id,
          format!("Provider component {} failed: {}", name, e.to_string()),
        );
      }
      let mut receiver = receiver.unwrap();
      let (tx, rx) = unbounded_channel();
      actix::spawn(async move {
        loop {
          trace!("Provider component {} waiting for output", name);
          let next = receiver.next().await;
          if next.is_none() {
            break;
          }

          let (port_name, msg) = next.unwrap();
          trace!("Native actor {} got output on port [{}]", name, port_name);
          match tx.send(NativeOutputReady {
            port: port_name.to_string(),
            payload: msg,
            invocation_id: invocation_id.to_string(),
          }) {
            Ok(_) => {
              trace!("Sent output to port '{}' ", port_name);
            }
            Err(e) => {
              error!("Error sending output on channel {}", e.to_string());
              break;
            }
          }
        }
      });
      InvocationResponse::stream(tx_id, rx)
    };
    ActorResult::reply_async(request.into_actor(self))
  }
}

#[cfg(test)]
mod test {

  use maplit::hashmap;
  use nkeys::KeyPair;
  use test_vino_provider::Provider;
  use vino_codec::messagepack::serialize;
  use vino_transport::MessageTransport;

  use super::*;
  use crate::dispatch::ComponentEntity;
  use crate::{
    Invocation,
    VinoEntity,
  };

  #[test_env_log::test(actix_rt::test)]
  async fn test_provider_component() -> Result<()> {
    let provider_component = ProviderComponent::default();
    let addr = provider_component.start();
    let provider = Provider::default();

    let hostkey = KeyPair::new_server();
    let host_id = KeyPair::new_server().public_key();
    let namespace = "test-namespace";

    addr
      .send(Initialize {
        name: "native-provider".to_string(),
        provider: Arc::new(Mutex::new(provider)),
        seed: hostkey.seed()?,
      })
      .await??;
    let user_data = "This is my payload";

    let payload = hashmap! {"input".to_string()=> serialize(user_data)?};

    let response = addr
      .send(Invocation {
        origin: VinoEntity::Test("test".to_string()),
        target: VinoEntity::Component(ComponentEntity {
          id: namespace.into(),
          reference: "hmmm".into(),
          name: "test-component".into(),
        }),
        msg: MessageTransport::MultiBytes(payload),
        id: Invocation::uuid(),
        tx_id: Invocation::uuid(),
        encoded_claims: "".to_string(),
        host_id,
      })
      .await?;
    let (_, mut rx) = response.to_stream()?;
    let next: NativeOutputReady = rx.recv().await.unwrap();
    let payload: String = next.payload.try_into()?;
    println!("response: {:?}", payload);
    assert_eq!(user_data, payload);

    Ok(())
  }
}
