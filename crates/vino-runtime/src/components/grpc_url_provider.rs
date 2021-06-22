use actix::fut::{
  err,
  ok,
};
use actix::prelude::*;
use futures::FutureExt;
use rpc::component_rpc_client::ComponentRpcClient;
use tokio::sync::mpsc::UnboundedSender;
use tonic::transport::Channel;
use vino_guest::OutputPayload;
use vino_rpc as rpc;
use vino_transport::{
  deserialize,
  serialize,
};

use crate::dispatch::{
  Invocation,
  InvocationResponse,
  MessagePayload,
};
use crate::error::VinoError;
use crate::Result;

#[derive(Default, Debug)]
pub struct GrpcUrlProvider {
  namespace: String,
  state: Option<State>,
  seed: String,
}

#[derive(Debug)]
struct State {
  pub(crate) client: ComponentRpcClient<Channel>,
  pub(crate) sender: UnboundedSender<MessagePayload>,
}

impl Actor for GrpcUrlProvider {
  type Context = Context<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    debug!("GRPC Provider started");
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {}
}

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
  pub(crate) namespace: String,
  pub(crate) address: String,
  pub(crate) signing_seed: String,
  pub(crate) sender: UnboundedSender<MessagePayload>,
}

impl Handler<Initialize> for GrpcUrlProvider {
  type Result = ResponseActFuture<Self, Result<()>>;

  fn handle(&mut self, msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {
    debug!("Native actor initialized");
    self.namespace = msg.namespace;
    self.seed = msg.signing_seed;
    let sender = msg.sender;

    let addr = msg.address;

    Box::pin(ComponentRpcClient::connect(addr).into_actor(self).then(
      move |client, provider, _ctx| match client {
        Ok(client) => {
          provider.state = Some(State { client, sender });
          ok(())
        }
        Err(e) => err(VinoError::ProviderError(format!(
          "Could not connect to Rpc Client in GrpcUrlProvider: {}",
          e
        ))),
      },
    ))
  }
}

impl Handler<Invocation> for GrpcUrlProvider {
  type Result = ResponseFuture<InvocationResponse>;

  fn handle(&mut self, msg: Invocation, _ctx: &mut Self::Context) -> Self::Result {
    debug!(
      "Native actor Invocation - From {} to {}",
      msg.origin.url(),
      msg.target.url()
    );
    let target_url = msg.target.url();
    let target = msg.target;
    let payload = msg.msg;
    let tx_id = msg.tx_id;
    let tx_id2 = tx_id.clone();
    let claims = msg.encoded_claims;
    let host_id = msg.host_id;

    let inv_id = msg.id;
    let state = self.state.as_ref().unwrap();
    // let seed = self.seed.clone();
    let mut client = state.client.clone();
    let sender = state.sender.clone();
    let origin = msg.origin;

    let fut = async move {
      let entity = target
        .into_component()
        .map_err(|_| "Provider received invalid invocation")?;
      debug!("Getting component: {}", entity);
      let payload = payload
        .into_multibytes()
        .map_err(|_| VinoError::ComponentError("Provider sent invalid payload".into()))?;
      debug!("Payload is : {:?}", payload);
      let payload = serialize(payload).map_err(|_| "Could not serialize input payload")?;

      debug!("making external request {}", target_url);

      let invocation = rpc::Invocation {
        origin: Some(rpc::Entity {
          name: origin.key(),
          kind: rpc::entity::EntityKind::Test.into(),
        }),
        target: Some(rpc::Entity {
          name: entity.name,
          kind: rpc::entity::EntityKind::Provider.into(),
        }),
        msg: Some(rpc::MessagePayload {
          kind: rpc::PayloadKind::MultiBytes.into(),
          value: payload,
        }),
        id: inv_id.to_string(),
        tx_id: tx_id.to_string(),
        encoded_claims: claims.to_string(),
        host_id: host_id.to_string(),
      };

      let stream = client.invoke(invocation).await?;
      actix::spawn(async move {
        let mut stream = stream.into_inner();
        loop {
          match stream.message().await {
            Ok(next) => {
              if next.is_none() {
                break;
              }
              let output = next.unwrap();

              // let kp = KeyPair::from_seed(&seed).unwrap();
              debug!(
                "Provider Component for invocation {} got output on port [{}]: result: {:?}",
                output.invocation_id, output.port, output.payload
              );
              let output_payload = output.payload.unwrap();
              let kind: rpc::PayloadKind = rpc::PayloadKind::from_i32(output_payload.kind).unwrap();
              debug!("Payload kind: {:?}", kind);
              let payload = match kind {
                rpc::PayloadKind::Error => OutputPayload::Error(
                  deserialize(&output_payload.value)
                    .unwrap_or_else(|_| "Can not deserialize output payload".to_string()),
                ),
                rpc::PayloadKind::Exception => OutputPayload::Exception(
                  deserialize(&output_payload.value)
                    .unwrap_or_else(|_| "Can not deserialize output payload".to_string()),
                ),
                rpc::PayloadKind::Invalid => {
                  OutputPayload::Error("Invalid payload kind".to_string())
                }
                rpc::PayloadKind::Test => OutputPayload::Test(
                  deserialize(&output_payload.value)
                    .unwrap_or_else(|_| "Invalid payload kind".to_string()),
                ),
                rpc::PayloadKind::MessagePack => OutputPayload::MessagePack(output_payload.value),
                rpc::PayloadKind::MultiBytes => {
                  OutputPayload::Error("Invalid payload kind".to_string())
                }
              };
              let result = sender.send(MessagePayload::from(payload));
              match result {
                Ok(_) => {
                  debug!("successfully sent output payload to receiver");
                }
                Err(e) => {
                  error!("error sending output payload to receiver: {}", e);
                }
              }
              // let _result = native_host_callback(kp, &inv_id, "", &output.port, &payload).unwrap();
            }
            Err(e) => {
              error!("Received error from GRPC Url Provider upstream: {} ", e);
              break;
            }
          }
        }
      });
      Ok!(InvocationResponse::success(tx_id, vec![],))
    };

    Box::pin(fut.then(|result| async {
      match result {
        Ok(invocation) => invocation,
        Err(e) => InvocationResponse::error(tx_id2, e.to_string()),
      }
    }))
  }
}

#[cfg(test)]
mod test {

  use std::net::{
    IpAddr,
    Ipv4Addr,
    SocketAddr,
  };
  use std::str::FromStr;

  use maplit::hashmap;
  use tokio::sync::mpsc::unbounded_channel;
  use tonic::transport::Server;
  use vino_provider_test::Provider;
  use vino_rpc::component_rpc_server::ComponentRpcServer;
  use vino_rpc::ComponentService;
  use vino_transport::serialize;

  use super::*;
  use crate::dispatch::ComponentEntity;
  use crate::VinoEntity;

  async fn make_grpc_server(provider: Provider) -> Result<()> {
    let addr: SocketAddr =
      SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()), 54321);

    debug!("Binding to {:?}", addr.to_string());

    let component_service = ComponentService::new(provider);

    let svc = ComponentRpcServer::new(component_service);

    Server::builder()
      .add_service(svc)
      .serve(addr)
      .await
      .unwrap();

    debug!("Server started");
    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn test_init() -> Result<()> {
    let init_handle = make_grpc_server(Provider::default());

    debug!("test_init");
    let (tx, rx) = unbounded_channel();
    let mut rx = rx;
    let work = async move {
      let grpc_provider = GrpcUrlProvider::start_default();
      grpc_provider
        .send(Initialize {
          namespace: "test".to_string(),
          address: "https://127.0.0.1:54321".to_string(),
          signing_seed: "seed".to_string(),
          sender: tx,
        })
        .await??;

      grpc_provider
        .send(Invocation {
          origin: VinoEntity::Test("grpc".to_string()),
          target: VinoEntity::Component(ComponentEntity {
            id: "test::DEADBEEF".to_string(),
            reference: "DEADBEEF".to_string(),
            name: "test-component".to_string(),
          }),
          msg: MessagePayload::MultiBytes(hashmap! {
            "input".to_string()=>serialize("test string payload")?
          }),
          id: Invocation::uuid(),
          tx_id: Invocation::uuid(),
          encoded_claims: "".to_string(),
          host_id: Invocation::uuid(),
        })
        .await?;
      Ok!(())
    };
    tokio::select! {
        _ = work => {
            debug!("Work complete");
        }
        _ = init_handle => {
            panic!("server died");
        }
    };
    let next = rx.recv().await;
    match next {
      Some(n) => {
        let result: OutputPayload = match n {
          MessagePayload::MessagePack(bytes) => deserialize(&bytes)?,
          _ => panic!("Unexpected payload"),
        };

        debug!("Got next : {:?}", result);
      }
      None => panic!("Nothing received"),
    }
    Ok(())
  }
}
