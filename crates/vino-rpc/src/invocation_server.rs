use std::str::FromStr;

use rpc::{
  ListResponse,
  StatsResponse,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{
  Response,
  Status,
};
use vino_transport::message_transport::MessageSignal;
use vino_transport::MessageTransport;

use crate::rpc::invocation_service_server::InvocationService;
use crate::rpc::message_kind::{
  Data,
  Kind,
  OutputSignal,
};
use crate::rpc::{
  stats_request,
  MessageKind,
  Output,
};
use crate::{
  convert_messagekind_map,
  rpc,
  RpcHandlerType,
};

/// An implementation of a GRPC server for implementers of [RpcHandler] like Vino providers.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct InvocationServer {
  /// The provider that will handle incoming requests
  #[derivative(Debug = "ignore")]
  pub provider: RpcHandlerType,
}

impl InvocationServer {
  /// Constructor
  #[must_use]
  pub fn new(provider: RpcHandlerType) -> Self {
    Self { provider }
  }

  /// Constructor that takes in a provider already wrapped in an Arc<Mutex<>>
  #[must_use]
  pub fn new_shared(provider: RpcHandlerType) -> Self {
    Self { provider }
  }
}

fn make_output(port: &str, inv_id: &str, payload: MessageTransport) -> Result<Output, Status> {
  match payload {
    MessageTransport::Invalid => Ok(Output {
      port: port.to_owned(),
      invocation_id: inv_id.to_owned(),
      payload: Some(MessageKind {
        kind: Kind::Invalid.into(),
        data: None,
      }),
    }),
    MessageTransport::Exception(msg) => Ok(Output {
      port: port.to_owned(),
      invocation_id: inv_id.to_owned(),
      payload: Some(MessageKind {
        kind: Kind::Exception.into(),
        data: Some(Data::Message(msg)),
      }),
    }),
    MessageTransport::Error(msg) => Ok(Output {
      port: port.to_owned(),
      invocation_id: inv_id.to_owned(),
      payload: Some(MessageKind {
        kind: Kind::Error.into(),
        data: Some(Data::Message(msg)),
      }),
    }),
    MessageTransport::MessagePack(bytes) => Ok(Output {
      port: port.to_owned(),
      invocation_id: inv_id.to_owned(),
      payload: Some(MessageKind {
        kind: Kind::MessagePack.into(),
        data: Some(Data::Messagepack(bytes)),
      }),
    }),
    MessageTransport::Test(_) => todo!(),
    MessageTransport::Signal(signal) => match signal {
      MessageSignal::Done => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(MessageKind {
          kind: Kind::Signal.into(),
          data: Some(Data::Signal(OutputSignal::Done.into())),
        }),
      }),
      MessageSignal::OpenBracket => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(MessageKind {
          kind: Kind::Signal.into(),
          data: Some(Data::Signal(OutputSignal::OpenBracket.into())),
        }),
      }),
      MessageSignal::CloseBracket => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(MessageKind {
          kind: Kind::Signal.into(),
          data: Some(Data::Signal(OutputSignal::CloseBracket.into())),
        }),
      }),
    },
    MessageTransport::Success(v) => match serde_json::to_string(&v) {
      Ok(json) => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(MessageKind {
          kind: Kind::Json.into(),
          data: Some(Data::Json(json)),
        }),
      }),
      Err(e) => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(MessageKind {
          kind: Kind::Error.into(),
          data: Some(Data::Message(e.to_string())),
        }),
      }),
    },
    MessageTransport::Json(json) => Ok(Output {
      port: port.to_owned(),
      invocation_id: inv_id.to_owned(),
      payload: Some(MessageKind {
        kind: Kind::Json.into(),
        data: Some(Data::Json(json)),
      }),
    }),
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
    let invocation = request.get_ref();
    let invocation_id = invocation.id.clone();
    let entity = vino_entity::Entity::from_str(&invocation.target);
    if let Err(e) = entity {
      tx.send(Err(Status::failed_precondition(e.to_string())))
        .await
        .unwrap();
    } else {
      let entity = entity.unwrap();
      let payload = convert_messagekind_map(&invocation.msg);
      debug!("Executing component {}", entity.url());
      let result = self.provider.invoke(entity, payload).await;
      tokio::spawn(async move {
        if let Err(e) = result {
          let message = e.to_string();
          tx.send(Err(Status::internal(message))).await.unwrap();
          return;
        }
        let mut receiver = result.unwrap();
        while let Some(next) = receiver.next().await {
          let port_name = next.port;
          let msg = next.payload;
          debug!("Got output on port {}", port_name);
          tx.send(make_output(&port_name, &invocation_id, msg))
            .await
            .unwrap();
        }
      });
    }

    Ok(Response::new(ReceiverStream::new(rx)))
  }

  async fn list(
    &self,
    _request: tonic::Request<rpc::ListRequest>,
  ) -> Result<Response<ListResponse>, Status> {
    trace!("Listing registered components from provider");
    let future = self.provider.get_list();
    let list = future.await.map_err(|e| Status::internal(e.to_string()))?;
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

    let kind = stats_request
      .kind
      .as_ref()
      .ok_or_else(|| Status::failed_precondition("No kind given"))?;

    let future = match kind {
      stats_request::Kind::All(_format) => self.provider.get_stats(None),
      stats_request::Kind::Component(comp) => self.provider.get_stats(Some(comp.name.clone())),
    };

    let stats = future.await.map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(StatsResponse {
      stats: stats.into_iter().map(From::from).collect(),
    }))
  }
}

// pub struct RpcWrapper {
//   provider: Arc<Mutex<dyn RpcHandler>>,
// }

// #[async_trait::async_trait]
// impl RpcHandler for RpcWrapper {
//   async fn invoke(
//     &self,
//     entity: vino_entity::Entity,
//     payload: vino_transport::TransportMap,
//   ) -> crate::RpcResult<crate::BoxedTransportStream> {
//     self.inner
//   }

//   async fn get_list(&self) -> crate::RpcResult<Vec<vino_types::signatures::HostedType>> {
//     todo!()
//   }

//   async fn get_stats(&self, id: Option<String>) -> crate::RpcResult<Vec<crate::Statistics>> {
//     todo!()
//   }
// }
