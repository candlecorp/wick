use std::collections::HashMap;
use std::convert::TryInto;

use actix::prelude::*;
use futures::FutureExt;
use rpc::invocation_service_client::InvocationServiceClient;
use tokio::sync::mpsc::unbounded_channel;
use tonic::transport::Channel;
use tracing::Instrument;
use tracing_actix::ActorInstrument;
use vino_component::v0::Payload;
use vino_component::Packet;
use vino_rpc::rpc::stats_request::Format;
use vino_rpc::rpc::{
  ListRequest,
  StatsRequest,
};
use vino_rpc::{
  rpc,
  HostedType,
};

use super::{
  ProviderMessage,
  ProviderResponse,
};
use crate::actix::ActorResult;
use crate::component_model::ComponentModel;
use crate::dispatch::{
  Invocation,
  InvocationResponse,
};
use crate::error::VinoError;
use crate::schematic::PushOutput;
use crate::Result;

#[derive(Default, Debug)]
pub struct GrpcUrlProvider {
  namespace: String,
  state: Option<State>,
  seed: String,
}

#[derive(Debug)]
struct State {
  client: InvocationServiceClient<Channel>,
  components: HashMap<String, ComponentModel>,
}

impl Actor for GrpcUrlProvider {
  type Context = Context<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    debug!("GRPC Provider started");
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {}
}

#[derive(Message, Debug)]
#[rtype(result = "Result<HashMap<String, ComponentModel>>")]
pub(crate) struct Initialize {
  pub(crate) namespace: String,
  pub(crate) address: String,
  pub(crate) signing_seed: String,
}

impl Handler<Initialize> for GrpcUrlProvider {
  type Result = ActorResult<Self, Result<HashMap<String, ComponentModel>>>;

  #[tracing::instrument(level = tracing::Level::DEBUG, name = "GRPC Init", skip(self, msg, ctx))]
  fn handle(&mut self, msg: Initialize, ctx: &mut Self::Context) -> Self::Result {
    debug!("GRPC Provider initialized");

    self.namespace = msg.namespace;
    self.seed = msg.signing_seed;

    let address = msg.address;
    let addr = ctx.address();
    let namespace = self.namespace.clone();

    let task = InvocationServiceClient::connect(address);
    let after = |client: std::result::Result<InvocationServiceClient<Channel>, _>| async move {
      match client {
        Ok(client) => {
          let metadata = addr
            .send(InitializeComponents {
              namespace: namespace.clone(),
              client: client.clone(),
            })
            .await??;
          Ok!((client, metadata))
        }
        Err(e) => Err(VinoError::GrpcUrlProviderError(format!(
          "Could not connect client: {}",
          e
        ))),
      }
    };

    let chain = task
      .then(after)
      .into_actor(self)
      .actor_instrument(debug_span!("actor"))
      .map(|result, this, _ctx| match result {
        Ok((client, metadata)) => {
          this.state = Some(State {
            client,
            components: metadata.clone(),
          });
          Ok(metadata)
        }
        Err(e) => Err(e),
      });

    ActorResult::reply_async(chain)
  }
}

#[derive(Message)]
#[rtype(result = "Result<HashMap<String, ComponentModel>>")]
pub(crate) struct InitializeComponents {
  namespace: String,
  client: InvocationServiceClient<Channel>,
}

impl Handler<InitializeComponents> for GrpcUrlProvider {
  type Result = ActorResult<Self, Result<HashMap<String, ComponentModel>>>;

  #[tracing::instrument(level = tracing::Level::DEBUG, name = "GRPC Init components", skip(self, msg, ctx))]
  fn handle(&mut self, msg: InitializeComponents, ctx: &mut Self::Context) -> Self::Result {
    trace!(
      "Initializing components for GRPC Url Provider '{}'",
      msg.namespace
    );
    let mut client = msg.client;
    let namespace = msg.namespace;
    let addr = ctx.address();

    let task = async move {
      let list = client
        .list(ListRequest {})
        .instrument(debug_span!("client.list"));
      debug!("Making list request");
      let list = list.await?;
      debug!("Component list: {:?}", list);
      let list = list.into_inner();

      let mut metadata: HashMap<String, ComponentModel> = HashMap::new();

      for item in list.component {
        metadata.insert(
          item.name.to_string(),
          ComponentModel {
            id: format!("{}::{}", namespace, item.name),
            name: item.name.to_string(),
            inputs: item.inputs.iter().map(|p| p.name.clone()).collect(),
            outputs: item.outputs.iter().map(|p| p.name.clone()).collect(),
            addr: addr.clone().recipient(),
          },
        );
      }
      Ok(metadata)
    };

    ActorResult::reply_async(
      task
        .into_actor(self)
        .actor_instrument(span!(tracing::Level::DEBUG, "actor")),
    )
  }
}

impl Handler<ProviderMessage> for GrpcUrlProvider {
  type Result = ActorResult<Self, Result<ProviderResponse>>;

  fn handle(&mut self, msg: ProviderMessage, ctx: &mut Self::Context) -> Self::Result {
    let state = self.state.as_ref().unwrap();
    let mut client = state.client.clone();
    let namespace = self.namespace.clone();
    let addr = ctx.address();

    let task = async move {
      returns!(ProviderResponse);
      match msg {
        ProviderMessage::Invoke(_invocation) => todo!(),
        ProviderMessage::List(_req) => {
          let list = client
            .list(ListRequest {})
            .instrument(debug_span!("client.list"))
            .await?;
          debug!("Component list: {:?}", list);
          let list = list.into_inner();

          let mut metadata: HashMap<String, ComponentModel> = HashMap::new();
          let mut hosted_types = vec![];

          for item in list.component {
            let id = format!("{}::{}", namespace, item.name);
            let input_names = item.inputs.iter().map(|p| p.name.clone()).collect();
            let output_names = item.outputs.iter().map(|p| p.name.clone()).collect();
            let model = ComponentModel {
              id,
              name: item.name.to_string(),
              inputs: input_names,
              outputs: output_names,
              addr: addr.clone().recipient(),
            };
            hosted_types.push(HostedType::Component(vino_rpc::Component {
              name: model.name.clone(),
              inputs: item.inputs.into_iter().map(From::from).collect(),
              outputs: item.outputs.into_iter().map(From::from).collect(),
            }));
            metadata.insert(item.name.to_string(), model);
          }

          Ok(ProviderResponse::List(hosted_types))
        }
        ProviderMessage::Statistics(_req) => {
          let stats = client
            .stats(StatsRequest {
              kind: Some(rpc::stats_request::Kind::All(Format {})),
            })
            .instrument(debug_span!("client.stats"))
            .await?;
          let stats = stats.into_inner();

          Ok(ProviderResponse::Stats(
            stats.stats.into_iter().map(From::from).collect(),
          ))
        }
      }
    };
    ActorResult::reply_async(task.into_actor(self))
  }
}

impl Handler<Invocation> for GrpcUrlProvider {
  type Result = ActorResult<Self, InvocationResponse>;

  #[tracing::instrument(level = tracing::Level::DEBUG, name = "GRPC Invocation", skip(self, msg, _ctx))]
  fn handle(&mut self, msg: Invocation, _ctx: &mut Self::Context) -> Self::Result {
    let state = self.state.as_ref().unwrap();
    let mut client = state.client.clone();
    let tx_id = msg.tx_id.clone();
    let tx_id2 = msg.tx_id.clone();
    let component = actix_ensure_ok!(msg
      .target
      .clone()
      .into_component()
      .map_err(|_e| InvocationResponse::error(tx_id.clone(), "Sent invalid entity".to_string())));

    let name = component.name;
    let inv_id = msg.id.to_string();
    let invocation: rpc::Invocation = actix_ensure_ok!(msg
      .try_into()
      .map_err(|_e| InvocationResponse::error(tx_id.clone(), "Sent invalid payload".to_string())));

    let request = async move {
      let invocation_id = inv_id.clone();
      let mut stream = client.invoke(invocation).await?.into_inner();
      let (tx, rx) = unbounded_channel();
      actix::spawn(async move {
        loop {
          trace!("Provider component {} waiting for output", name);
          let next = stream.message().await;
          if let Err(e) = next {
            let msg = format!("Error during GRPC stream: {}", e);
            error!("{}", msg);
            match tx.send(PushOutput {
              port: crate::COMPONENT_ERROR.to_string(),
              payload: Packet::V0(Payload::Error(msg)),
              invocation_id: invocation_id.to_string(),
            }) {
              Ok(_) => {
                trace!("Sent error to upstream, closing connection.");
              }
              Err(e) => {
                error!("Error sending output on channel {}", e.to_string());
              }
            }
            break;
          }
          let next = next.unwrap();

          if next.is_none() {
            break;
          }
          let output = next.unwrap();
          let payload = output.payload;
          if payload.is_none() {
            let msg = "Received response but no payload";
            error!("{}", msg);
            match tx.send(PushOutput {
              port: crate::COMPONENT_ERROR.to_string(),
              payload: Packet::V0(Payload::Error(msg.to_string())),
              invocation_id: invocation_id.to_string(),
            }) {
              Ok(_) => {
                trace!("Sent error to upstream");
              }
              Err(e) => {
                error!(
                  "Error sending output on channel {}. Closing connection.",
                  e.to_string()
                );
                break;
              }
            }
            continue;
          }
          let port_name = output.port;
          trace!("Native actor {} got output on port [{}]", name, port_name);
          match tx.send(PushOutput {
            port: port_name.to_string(),
            payload: payload.unwrap().into(),
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
      Ok!(InvocationResponse::stream(tx_id, rx))
    };
    ActorResult::reply_async(
      request
        .into_actor(self)
        .actor_instrument(span!(tracing::Level::DEBUG, "actor"))
        .map(|result, _, _| match result {
          Ok(response) => response,
          Err(e) => InvocationResponse::error(tx_id2, format!("GRPC Invocation failed: {}", e)),
        }),
    )
  }
}

#[cfg(test)]
mod test {

  use std::net::{
    IpAddr,
    Ipv4Addr,
    SocketAddr,
  };
  use std::time::Duration;

  use actix::clock::sleep;
  use actix_rt::task::JoinHandle;
  use maplit::hashmap;
  use test_vino_provider::Provider;
  use tonic::transport::Server;
  use tracing::Instrument;
  use vino_codec::messagepack::serialize;
  use vino_rpc::rpc::invocation_service_server::InvocationServiceServer;
  use vino_rpc::InvocationServer;
  use vino_transport::MessageTransport;

  use super::*;
  use crate::components::{
    bind_new_socket,
    make_grpc_server,
  };
  use crate::dispatch::ComponentEntity;
  use crate::VinoEntity;

  #[test_env_log::test(actix_rt::test)]
  #[instrument]
  async fn test_initialize() -> Result<()> {
    let socket = bind_new_socket()?;
    let port = socket.local_addr()?.port();
    let init_handle = make_grpc_server(socket, Provider::default());
    let user_data = "test string payload";

    let work = async move {
      sleep(Duration::from_secs(1)).await;
      let grpc_provider = GrpcUrlProvider::start_default();
      grpc_provider
        .send(Initialize {
          namespace: "test".to_string(),
          address: format!("https://127.0.0.1:{}", port),
          signing_seed: "seed".to_string(),
        })
        .await??;
      debug!("Initialized");

      let response = grpc_provider
        .send(Invocation {
          origin: VinoEntity::Test("grpc".to_string()),
          target: VinoEntity::Component(ComponentEntity {
            id: "test::REFERENCE".to_string(),
            reference: "REFERENCE".to_string(),
            name: "test-component".to_string(),
          }),
          msg: MessageTransport::MultiBytes(hashmap! {
            "input".to_string()=>serialize(user_data)?
          }),
          id: Invocation::uuid(),
          tx_id: Invocation::uuid(),
          encoded_claims: "".to_string(),
          host_id: Invocation::uuid(),
        })
        .await?;
      Ok!(response)
    }
    .instrument(tracing::info_span!("task"));
    tokio::select! {
        res = work => {
            debug!("Work complete");
            match res {
              Ok(response)=>{
                let (_, mut rx) = response.to_stream()?;
                let next: PushOutput = rx.recv().await.unwrap();
                let payload: String = next.payload.try_into()?;
                assert_eq!(user_data, payload);              },
              Err(e)=>{
                panic!("task failed: {}", e);
              }
            }
        }
        _ = init_handle => {
            panic!("server died");
        }
    };

    Ok(())
  }
}
