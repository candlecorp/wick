use std::sync::Arc;

use actix::prelude::*;
use tokio::sync::Mutex;
use vino_rpc::RpcHandler;

use super::{
  ProviderMessage,
  ProviderResponse,
};
use crate::actix::ActorResult;
use crate::Result;

#[derive(Debug)]
pub struct NativeProvider {
  state: Option<Arc<Mutex<State>>>,
}

impl Default for NativeProvider {
  fn default() -> Self {
    Self { state: None }
  }
}

struct State {
  provider: Box<dyn RpcHandler>,
}

impl std::fmt::Debug for State {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Receiver")
      .field("provider", &"<skipped>".to_string())
      .finish()
  }
}

impl Actor for NativeProvider {
  type Context = Context<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    trace!("Native actor started");
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {}
}

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
  pub(crate) name: String,
  pub(crate) host_id: String,
  pub(crate) seed: String,
}

impl Handler<Initialize> for NativeProvider {
  type Result = Result<()>;

  fn handle(&mut self, _msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {
    trace!("Native actor initialized");
    self.state = Some(Arc::new(Mutex::new(State {
      provider: Box::new(vino_native_provider::Provider::default()),
    })));
    Ok(())
  }
}

impl Handler<ProviderMessage> for NativeProvider {
  type Result = ActorResult<Self, Result<ProviderResponse>>;

  fn handle(&mut self, msg: ProviderMessage, _ctx: &mut Self::Context) -> Self::Result {
    let state = self.state.as_ref().unwrap().clone();

    let task = async move {
      let state = state.lock().await;
      returns!(ProviderResponse);
      match msg {
        ProviderMessage::Invoke(_invocation) => todo!(),
        ProviderMessage::List(_req) => {
          let list = state.provider.list_registered().await?;
          Ok(ProviderResponse::ListResponse(list))
        }
        ProviderMessage::Statistics(_req) => todo!(),
      }
    };
    ActorResult::reply_async(task.into_actor(self))
  }
}

#[cfg(test)]
mod test {

  use nkeys::KeyPair;

  use super::*;
  use crate::components::ListRequest;
  use crate::Invocation;

  #[test_env_log::test(actix_rt::test)]
  async fn test_native_provider_list() -> Result<()> {
    let provider = NativeProvider::default();
    let addr = provider.start();

    let hostkey = KeyPair::new_server();
    let host_id = KeyPair::new_server().public_key();
    let tx_id = Invocation::uuid();

    addr
      .send(Initialize {
        name: "native-provider".to_string(),
        host_id: host_id.to_string(),
        seed: hostkey.seed()?,
      })
      .await??;

    let response = addr
      .send(super::super::ProviderMessage::List(ListRequest {}))
      .await??;
    println!("response: {:?}", response);
    let list = response.into_list_response()?;
    assert_eq!(list.len(), 4);

    Ok(())
  }
}
