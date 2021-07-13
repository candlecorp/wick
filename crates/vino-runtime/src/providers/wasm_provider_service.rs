use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex;
use vino_rpc::{
  HostedType,
  RpcHandler,
};

use super::{
  ProviderRequest,
  ProviderResponse,
};
use crate::dev::prelude::*;
use crate::error::ComponentError;
type Result<T> = std::result::Result<T, ComponentError>;

#[derive(Debug)]
pub(crate) struct NetworkProviderService {
  state: Option<State>,
}

impl Default for NetworkProviderService {
  fn default() -> Self {
    Self { state: None }
  }
}

#[derive(Derivative)]
#[derivative(Debug)]
struct State {
  #[derivative(Debug = "ignore")]
  provider: Arc<Mutex<dyn RpcHandler>>,
  namespace: String,
}

impl Actor for NetworkProviderService {
  type Context = Context<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    trace!("Native provider actor started");
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {}
}

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
  pub(crate) namespace: String,
  pub(crate) provider: Arc<Mutex<dyn RpcHandler>>,
}

impl Handler<Initialize> for NetworkProviderService {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {
    trace!("Network provider initializing for '{}'", msg.namespace);
    self.state = Some(State {
      provider: msg.provider,
      namespace: msg.namespace,
    });
    ActorResult::reply(Ok(()))
  }
}

#[derive(Message)]
#[rtype(result = "Result<HashMap<String, ComponentModel>>")]
pub(crate) struct InitializeComponents;

impl Handler<InitializeComponents> for NetworkProviderService {
  type Result = ActorResult<Self, Result<HashMap<String, ComponentModel>>>;

  fn handle(&mut self, _msg: InitializeComponents, _ctx: &mut Self::Context) -> Self::Result {
    let state = self.state.as_ref().unwrap();
    let provider = state.provider.clone();
    let namespace = state.namespace.clone();
    trace!("Initializing components for '{}'", namespace);

    let task = async move {
      let provider = provider.lock().await;
      let list = provider.list_registered().await?;
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

impl Handler<Invocation> for NetworkProviderService {
  type Result = ActorResult<Self, InvocationResponse>;

  fn handle(&mut self, msg: Invocation, _ctx: &mut Self::Context) -> Self::Result {
    let provider = self.state.as_ref().unwrap().provider.clone();
    let tx_id = msg.tx_id.clone();
    let component = msg.target;
    let message = actix_ensure_ok!(msg
      .msg
      .into_multibytes()
      .map_err(|_e| InvocationResponse::error(tx_id.clone(), "Sent invalid payload".to_owned())));
    let url = component.url();
    let inv_id = msg.id;

    let request = async move {
      let provider = provider.lock().await;
      let invocation_id = inv_id.clone();
      let receiver = provider.request(inv_id.clone(), component, message).await;
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
          trace!("Provider component {} waiting for output", url);
          let next = receiver.next().await;
          if next.is_none() {
            break;
          }

          let output = next.unwrap();
          trace!("Native actor {} got output on port [{}]", url, output.port);
          match tx.send(OutputPacket {
            port: output.port.clone(),
            payload: output.packet,
            invocation_id: invocation_id.clone(),
          }) {
            Ok(_) => {
              trace!("Sent output to port '{}' ", output.port);
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

impl Handler<ProviderRequest> for NetworkProviderService {
  type Result = ActorResult<Self, Result<ProviderResponse>>;

  fn handle(&mut self, msg: ProviderRequest, _ctx: &mut Self::Context) -> Self::Result {
    let state = self.state.as_ref().unwrap();
    let provider = state.provider.clone();

    let task = async move {
      let provider = provider.lock().await;
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
  // Tests found in integration tests
}
