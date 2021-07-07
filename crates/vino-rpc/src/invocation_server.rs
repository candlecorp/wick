use std::str::FromStr;
use std::sync::Arc;

use rpc::{
  ListResponse,
  StatsResponse,
};
use tokio::sync::{
  mpsc,
  Mutex,
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{
  Response,
  Status,
};
use vino_component::Packet;

use crate::rpc::invocation_service_server::InvocationService;
use crate::rpc::output_kind::Data;
use crate::rpc::{
  stats_request,
  Output,
  OutputKind,
  OutputSignal,
};
use crate::{
  rpc,
  RpcHandler,
};

/// An implementation of a GRPC server for implementers of [RpcHandler] like Vino providers.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct InvocationServer {
  /// The provider that will handle incoming requests
  #[derivative(Debug = "ignore")]
  pub provider: Arc<Mutex<dyn RpcHandler>>,
}

impl InvocationServer {
  /// Constructor
  pub fn new<T>(provider: T) -> Self
  where
    T: RpcHandler + 'static,
  {
    Self {
      provider: Arc::new(Mutex::new(provider)),
    }
  }

  /// Constructor that takes in a provider already wrapped in an Arc<Mutex<>>
  #[must_use]
  pub fn new_shared<T>(provider: Arc<Mutex<T>>) -> Self
  where
    T: RpcHandler + 'static,
  {
    Self { provider }
  }
}

fn make_output(port: &str, inv_id: &str, payload: Packet) -> Result<Output, Status> {
  match payload {
    Packet::V0(v) => match v {
      vino_component::v0::Payload::Invalid => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(OutputKind {
          data: Some(Data::Invalid(true)),
        }),
      }),
      vino_component::v0::Payload::Exception(msg) => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(OutputKind {
          data: Some(Data::Exception(msg)),
        }),
      }),
      vino_component::v0::Payload::Error(msg) => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(OutputKind {
          data: Some(Data::Error(msg)),
        }),
      }),
      vino_component::v0::Payload::MessagePack(bytes) => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(OutputKind {
          data: Some(Data::Messagepack(bytes)),
        }),
      }),
      vino_component::v0::Payload::Close => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(OutputKind {
          data: Some(Data::Signal(OutputSignal::Close.into())),
        }),
      }),
      vino_component::v0::Payload::OpenBracket => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(OutputKind {
          data: Some(Data::Signal(OutputSignal::OpenBracket.into())),
        }),
      }),
      vino_component::v0::Payload::CloseBracket => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(OutputKind {
          data: Some(Data::Signal(OutputSignal::CloseBracket.into())),
        }),
      }),
    },
  }
}

#[tonic::async_trait]
impl InvocationService for InvocationServer {
  type InvokeStream = ReceiverStream<Result<Output, Status>>;

  async fn invoke(
    &self,
    request: tonic::Request<rpc::Invocation>,
  ) -> Result<Response<Self::InvokeStream>, Status> {
    debug!("Invocation = {:?}", request);

    let (tx, rx) = mpsc::channel(4);
    let provider = self.provider.clone();

    tokio::spawn(async move {
      let invocation = request.get_ref();
      let provider = provider.lock().await;
      let invocation_id = invocation.id.clone();
      let entity = vino_entity::Entity::from_str(&invocation.target);
      if let Err(e) = entity {
        tx.send(Err(Status::failed_precondition(e.to_string())))
          .await
          .unwrap();
        return;
      }
      let entity = entity.unwrap();
      let payload = invocation.msg.clone();
      debug!("Executing component {}", entity.url());
      match &mut provider
        .request(invocation_id.clone(), entity, payload)
        .await
      {
        Ok(receiver) => {
          while let Some(next) = receiver.next().await {
            let port_name = next.port;
            let msg = next.packet;
            debug!("Got output on port {}", port_name);
            tx.send(make_output(&port_name, &invocation_id, msg))
              .await
              .unwrap();
          }
        }
        Err(e) => {
          tx.send(Err(Status::internal(e.to_string()))).await.unwrap();
        }
      };
    });

    Ok(Response::new(ReceiverStream::new(rx)))
  }

  async fn list(
    &self,
    _request: tonic::Request<rpc::ListRequest>,
  ) -> Result<Response<ListResponse>, Status> {
    let provider = self.provider.lock().await;
    trace!("Listing registered components from provider");
    let list = provider
      .list_registered()
      .await
      .map_err(|e| Status::internal(e.to_string()))?;
    trace!("Server: list is {:?}", list);
    let response = ListResponse {
      components: list.into_iter().map(From::from).collect(),
    };

    Ok(Response::new(response))
  }

  async fn stats(
    &self,
    request: tonic::Request<rpc::StatsRequest>,
  ) -> Result<Response<StatsResponse>, Status> {
    let stats_request = request.get_ref();

    let provider = self.provider.lock().await;
    let kind = stats_request
      .kind
      .as_ref()
      .ok_or_else(|| Status::failed_precondition("No kind given"))?;

    let list = match kind {
      stats_request::Kind::All(_format) => provider
        .report_statistics(None)
        .await
        .map_err(|e| Status::internal(e.to_string()))?,
      stats_request::Kind::Component(comp) => provider
        .report_statistics(Some(comp.name.clone()))
        .await
        .map_err(|e| Status::internal(e.to_string()))?,
    };

    Ok(Response::new(StatsResponse {
      stats: list.into_iter().map(From::from).collect(),
    }))
  }
}
