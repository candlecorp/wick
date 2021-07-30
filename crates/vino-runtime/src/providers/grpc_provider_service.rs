use std::collections::HashMap;
use std::convert::TryInto;

use rpc::invocation_service_client::InvocationServiceClient;
use tokio::sync::mpsc::unbounded_channel;
use tonic::transport::Channel;
use tracing_actix::ActorInstrument;
use vino_rpc::rpc;
use vino_rpc::rpc::stats_request::Format;
use vino_rpc::rpc::{
  ListRequest,
  StatsRequest,
};

use super::{
  ProviderRequest,
  ProviderResponse,
};
use crate::dev::prelude::*;
type Result<T> = std::result::Result<T, ProviderError>;

static PREFIX: &str = "PRV:GRPC";

#[derive(Default, Debug)]
pub(crate) struct GrpcProviderService {
  prefix: String,
  namespace: String,
  state: Option<State>,
  seed: String,
}

#[derive(Debug)]
struct State {
  client: InvocationServiceClient<Channel>,
  components: HashMap<String, ComponentModel>,
}

impl Actor for GrpcProviderService {
  type Context = Context<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    trace!("{}:Service:Start", PREFIX);
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

impl Handler<Initialize> for GrpcProviderService {
  type Result = ActorResult<Self, Result<HashMap<String, ComponentModel>>>;

  #[tracing::instrument(level = tracing::Level::DEBUG, name = "GRPC Init", skip(self, msg, ctx))]
  fn handle(&mut self, msg: Initialize, ctx: &mut Self::Context) -> Self::Result {
    trace!("{}:Init:{}", PREFIX, msg.namespace);

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
          Ok((client, metadata))
        }
        Err(e) => Err(ProviderError::GrpcUrlProviderError(format!(
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
        Err(e) => log_err!(e),
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

impl Handler<InitializeComponents> for GrpcProviderService {
  type Result = ActorResult<Self, Result<HashMap<String, ComponentModel>>>;

  #[tracing::instrument(level = tracing::Level::DEBUG, name = "GRPC Init components", skip(self, msg, _ctx))]
  fn handle(&mut self, msg: InitializeComponents, _ctx: &mut Self::Context) -> Self::Result {
    trace!("{}:InitComponents:[NS:{}]", PREFIX, self.namespace);

    let mut client = msg.client;
    let namespace = msg.namespace;

    let task = async move {
      let list = client
        .list(ListRequest {})
        .instrument(debug_span!("client.list"));
      debug!("Making list request");
      let list = list.await?;
      let list = list.into_inner();

      let mut metadata: HashMap<String, ComponentModel> = HashMap::new();

      for item in list.components {
        metadata.insert(
          item.name.clone(),
          ComponentModel {
            namespace: namespace.clone(),
            name: item.name.clone(),
            inputs: item.inputs.into_iter().map(From::from).collect(),
            outputs: item.outputs.into_iter().map(From::from).collect(),
          },
        );
      }
      for (name, model) in &metadata {
        debug!("Component {} registering with metadata: {:?}", name, model);
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

impl Handler<ProviderRequest> for GrpcProviderService {
  type Result = ActorResult<Self, Result<ProviderResponse>>;

  fn handle(&mut self, msg: ProviderRequest, _ctx: &mut Self::Context) -> Self::Result {
    let state = self.state.as_ref().unwrap();
    let mut client = state.client.clone();

    let task = async move {
      match msg {
        ProviderRequest::Invoke(_invocation) => todo!(),
        ProviderRequest::List(_req) => {
          let list = client
            .list(ListRequest {})
            .instrument(debug_span!("client.list"))
            .await?;
          debug!("Component list: {:?}", list);
          let list = list.into_inner();

          let mut hosted_types = vec![];

          for item in list.components {
            hosted_types.push(HostedType::Component(ComponentSignature {
              name: item.name.clone(),
              inputs: item.inputs.into_iter().map(From::from).collect(),
              outputs: item.outputs.into_iter().map(From::from).collect(),
            }));
          }

          Ok(ProviderResponse::List(hosted_types))
        }
        ProviderRequest::Statistics(_req) => {
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

impl Handler<Invocation> for GrpcProviderService {
  type Result = ActorResult<Self, InvocationResponse>;

  #[tracing::instrument(level = tracing::Level::DEBUG, name = "GRPC Invocation", skip(self, msg, _ctx))]
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
    let mut client = state.client.clone();
    let tx_id = msg.tx_id.clone();
    let tx_id2 = msg.tx_id.clone();
    let url = msg.target.url();

    let invocation: rpc::Invocation =
      actix_ensure_ok!(msg.try_into().map_err(|_e| InvocationResponse::error(
        tx_id.clone(),
        "GRPC provider sent invalid payload".to_owned()
      )));

    let request = async move {
      let mut stream = client.invoke(invocation).await?.into_inner();
      let (tx, rx) = unbounded_channel();
      actix::spawn(async move {
        loop {
          trace!("{}:[NS:{}]:{}:WAIT", PREFIX, ns, url);
          let next = stream.message().await;
          if let Err(e) = next {
            let msg = format!("Error during GRPC stream: {}", e);
            error!("{}", msg);
            match tx.send(InvocationTransport {
              port: crate::COMPONENT_ERROR.to_owned(),
              payload: MessageTransport::Error(msg),
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
          let output = match next.unwrap() {
            Some(v) => v,
            None => break,
          };

          let payload = output.payload;
          if payload.is_none() {
            let msg = "Received response but no payload";
            error!("{}", msg);
            match tx.send(InvocationTransport {
              port: crate::COMPONENT_ERROR.to_owned(),
              payload: MessageTransport::Error(msg.to_owned()),
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
          trace!("{}:[NS:{}]:{}:PORT:{}:RECV", PREFIX, ns, url, output.port);

          match tx.send(InvocationTransport {
            port: output.port.clone(),
            payload: payload.unwrap().into(),
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

  use std::time::Duration;

  use actix::clock::sleep;
  use test_vino_provider::Provider;
  use vino_rpc::{
    bind_new_socket,
    make_rpc_server,
  };

  use super::*;
  use crate::test::prelude::assert_eq;
  type Result<T> = super::Result<T>;

  #[test_env_log::test(actix_rt::test)]
  #[instrument]
  async fn test_initialize() -> Result<()> {
    let socket = bind_new_socket()?;
    let port = socket.local_addr()?.port();
    let init_handle = make_rpc_server(socket, Provider::default());
    let user_data = "test string payload";

    let work = async move {
      sleep(Duration::from_secs(1)).await;
      let grpc_provider = GrpcProviderService::start_default();
      grpc_provider
        .send(Initialize {
          namespace: "test".to_owned(),
          address: format!("https://127.0.0.1:{}", port),
          signing_seed: "seed".to_owned(),
        })
        .await??;
      debug!("Initialized");

      let response = grpc_provider
        .send(Invocation {
          origin: Entity::test("grpc"),
          target: Entity::component("test-component"),
          msg: transport_map! {
            "input"=>user_data
          },
          id: get_uuid(),
          tx_id: get_uuid(),
          network_id: get_uuid(),
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
                let next: InvocationTransport = rx.next().await.unwrap();
                let payload: String = next.payload.try_into()?;
                assert_eq!(payload, format!("TEST: {}", user_data));
              },
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
