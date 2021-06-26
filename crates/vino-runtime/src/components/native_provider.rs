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

#[derive(Derivative)]
#[derivative(Debug)]
struct State {
  #[derivative(Debug = "ignore")]
  provider: Arc<Mutex<dyn RpcHandler>>,
}

impl Actor for NativeProvider {
  type Context = Context<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    trace!("Native provider actor started");
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {}
}

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
  pub(crate) name: String,
  pub(crate) provider: Arc<Mutex<dyn RpcHandler>>,
}

impl Handler<Initialize> for NativeProvider {
  type Result = Result<()>;

  fn handle(&mut self, msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {
    trace!("Native provider initialized for '{}'", msg.name);
    self.state = Some(Arc::new(Mutex::new(State {
      provider: msg.provider,
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
      let provider = state.provider.as_ref().lock().await;
      returns!(ProviderResponse);
      match msg {
        ProviderMessage::Invoke(_invocation) => todo!(),
        ProviderMessage::List(_req) => {
          let list = provider.list_registered().await?;
          Ok(ProviderResponse::List(list))
        }
        ProviderMessage::Statistics(_req) => {
          let stats = provider.report_statistics(None).await?;
          Ok(ProviderResponse::Stats(stats))
        }
      }
    };
    ActorResult::reply_async(task.into_actor(self))
  }
}

#[cfg(test)]
mod test {

  use super::*;
  use crate::components::ListRequest;

  #[test_env_log::test(actix_rt::test)]
  async fn test_native_provider_list() -> Result<()> {
    let provider = NativeProvider::default();
    let addr = provider.start();

    addr
      .send(Initialize {
        name: "native-provider".to_string(),
        provider: Arc::new(Mutex::new(vino_native_provider::Provider::default())),
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
