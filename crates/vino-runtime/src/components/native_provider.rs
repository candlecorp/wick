use std::collections::HashMap;
use std::sync::Arc;

use actix::prelude::*;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use vino_rpc::{
  HostedType,
  RpcHandler,
};

use super::{
  ProviderRequest,
  ProviderResponse,
};
use crate::actix::ActorResult;
use crate::component_model::ComponentModel;
use crate::prelude::*;
use crate::schematic::PushOutput;

#[derive(Debug)]
pub struct NativeProvider {
  state: Option<State>,
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
#[rtype(result = "Result<HashMap<String, ComponentModel>>")]
pub(crate) struct Initialize {
  pub(crate) namespace: String,
  pub(crate) provider: Arc<Mutex<dyn RpcHandler>>,
}

impl Handler<Initialize> for NativeProvider {
  type Result = ActorResult<Self, Result<HashMap<String, ComponentModel>>>;

  fn handle(&mut self, msg: Initialize, ctx: &mut Self::Context) -> Self::Result {
    trace!("Native provider initialized for '{}'", msg.namespace);
    let provider = msg.provider.clone();

    self.state = Some(State {
      provider: msg.provider,
    });
    let addr = ctx.address();
    let init_components = InitializeComponents {
      namespace: msg.namespace,
      provider,
    };
    let task = async move { addr.send(init_components).await? }.into_actor(self);
    ActorResult::reply_async(task)
  }
}

#[derive(Message)]
#[rtype(result = "Result<HashMap<String, ComponentModel>>")]
pub(crate) struct InitializeComponents {
  namespace: String,
  provider: Arc<Mutex<dyn RpcHandler>>,
}

impl Handler<InitializeComponents> for NativeProvider {
  type Result = ActorResult<Self, Result<HashMap<String, ComponentModel>>>;

  fn handle(&mut self, msg: InitializeComponents, _ctx: &mut Self::Context) -> Self::Result {
    trace!("Initializing components '{}'", msg.namespace);
    let provider = msg.provider;
    let namespace = msg.namespace;

    let task = async move {
      let provider = provider.lock().await;
      let list = provider.list_registered().await?;
      drop(provider);

      let mut metadata: HashMap<String, ComponentModel> = HashMap::new();

      for item in list {
        match item {
          HostedType::Component(component) => {
            metadata.insert(
              component.name.to_string(),
              ComponentModel {
                id: format!("{}::{}", namespace, component.name),
                name: component.name,
                inputs: component.inputs.into_iter().map(|p| p.name).collect(),
                outputs: component.outputs.into_iter().map(|p| p.name).collect(),
              },
            );
          }
          HostedType::Schematic(_) => panic!("Unimplemented"),
        }
      }
      Ok(metadata)
    };

    ActorResult::reply_async(task.into_actor(self))
  }
}

impl Handler<Invocation> for NativeProvider {
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
          match tx.send(PushOutput {
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

impl Handler<ProviderRequest> for NativeProvider {
  type Result = ActorResult<Self, Result<ProviderResponse>>;

  fn handle(&mut self, msg: ProviderRequest, _ctx: &mut Self::Context) -> Self::Result {
    let state = self.state.as_ref().unwrap();
    let provider = state.provider.clone();

    let task = async move {
      let provider = provider.lock().await;
      returns!(ProviderResponse);
      match msg {
        ProviderRequest::Invoke(_invocation) => todo!(),
        ProviderRequest::List(_req) => {
          let list = provider.list_registered().await?;
          Ok(ProviderResponse::List(list))
        }
        ProviderRequest::Statistics(_req) => {
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

  use maplit::hashmap;
  use nkeys::KeyPair;
  use vino_codec::messagepack::serialize;

  use super::*;
  use crate::components::ListRequest;
  use crate::dispatch::ComponentEntity;
  use crate::VinoEntity;

  #[test_env_log::test(actix_rt::test)]
  async fn test_native_provider_list() -> Result<()> {
    let provider = NativeProvider::default();
    let addr = provider.start();

    addr
      .send(Initialize {
        namespace: "native-provider".to_string(),
        provider: Arc::new(Mutex::new(vino_native_provider::Provider::default())),
      })
      .await??;

    let response = addr
      .send(super::super::ProviderRequest::List(ListRequest {}))
      .await??;
    println!("response: {:?}", response);
    let list = response.into_list_response()?;
    assert_eq!(list.len(), 4);

    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn test_provider_component() -> Result<()> {
    let provider = NativeProvider::default();
    let addr = provider.start();
    let hostkey = KeyPair::new_server();
    let host_id = hostkey.public_key();
    let namespace = "test-namespace";
    addr
      .send(Initialize {
        namespace: "native-provider".to_string(),
        provider: Arc::new(Mutex::new(vino_native_provider::Provider::default())),
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
          name: "log".into(),
        }),
        msg: MessageTransport::MultiBytes(payload),
        id: Invocation::uuid(),
        tx_id: Invocation::uuid(),
        encoded_claims: "".to_string(),
        host_id,
      })
      .await?;

    debug!("Response {:#?}", response);
    let (_, mut rx) = response.to_stream()?;
    let next: PushOutput = rx.recv().await.unwrap();
    let payload: String = next.payload.try_into()?;
    assert_eq!(user_data, payload);

    Ok(())
  }
}
