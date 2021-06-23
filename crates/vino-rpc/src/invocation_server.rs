use std::sync::Arc;

use serde::Serialize;
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
use vino_codec::messagepack::serialize;
use vino_component::Output as ComponentOutput;

use crate::rpc::invocation_service_server::InvocationService;
use crate::rpc::output_kind::Data;
use crate::rpc::{
  Output,
  OutputKind,
};
use crate::RpcHandler;
pub struct InvocationServer {
  pub provider: Arc<Mutex<dyn crate::RpcHandler>>,
}

impl InvocationServer {
  pub fn new<T>(provider: T) -> Self
  where
    T: RpcHandler + 'static,
  {
    Self {
      provider: Arc::new(Mutex::new(provider)),
    }
  }
  pub fn new_shared<T>(provider: Arc<Mutex<T>>) -> Self
  where
    T: RpcHandler + 'static,
  {
    Self { provider }
  }
}

pub fn make_output<T>(
  port: &str,
  inv_id: &str,
  payload: ComponentOutput<T>,
) -> Result<Output, Status>
where
  T: Serialize + Send,
{
  match payload {
    ComponentOutput::V0(v) => match v {
      vino_component::v0::Payload::Invalid => Ok(Output {
        port: port.to_string(),
        invocation_id: inv_id.to_string(),
        payload: Some(OutputKind {
          data: Some(Data::Invalid(true)),
        }),
      }),
      vino_component::v0::Payload::Serializable(value) => match serialize(value) {
        Ok(bytes) => Ok(Output {
          port: port.to_string(),
          invocation_id: inv_id.to_string(),
          payload: Some(OutputKind {
            data: Some(Data::Messagepack(bytes)),
          }),
        }),
        Err(_) => Err(Status::failed_precondition("Could not serialize payload")),
      },
      vino_component::v0::Payload::Exception(msg) => Ok(Output {
        port: port.to_string(),
        invocation_id: inv_id.to_string(),
        payload: Some(OutputKind {
          data: Some(Data::Exception(msg)),
        }),
      }),
      vino_component::v0::Payload::Error(msg) => Ok(Output {
        port: port.to_string(),
        invocation_id: inv_id.to_string(),
        payload: Some(OutputKind {
          data: Some(Data::Error(msg)),
        }),
      }),
      vino_component::v0::Payload::MessagePack(bytes) => Ok(Output {
        port: port.to_string(),
        invocation_id: inv_id.to_string(),
        payload: Some(OutputKind {
          data: Some(Data::Messagepack(bytes)),
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
    request: tonic::Request<crate::rpc::Invocation>,
  ) -> Result<tonic::Response<Self::InvokeStream>, tonic::Status> {
    debug!("Invocation = {:?}", request);

    let (tx, rx) = mpsc::channel(4);
    let provider = self.provider.clone();

    tokio::spawn(async move {
      let invocation = request.get_ref();
      let provider = provider.lock().await;
      let invocation_id = invocation.id.to_string();

      if invocation.target.is_none() {
        tx.send(Err(Status::failed_precondition("No target specified")))
          .await
          .unwrap();
        return;
      }
      let target = invocation.target.as_ref().unwrap();
      let payload = invocation.msg.clone();
      debug!("Executing component {}", target.name.to_string());
      match &mut provider
        .request(invocation_id.to_string(), target.name.to_string(), payload)
        .await
      {
        Ok(receiver) => {
          while let Some((port_name, msg)) = receiver.next().await {
            debug!("got output on port {}", port_name);
            tx.send(make_output(&port_name, &invocation_id, msg))
              .await
              .unwrap();
          }
        }
        Err(e) => {
          tx.send(Err(Status::aborted(e.to_string()))).await.unwrap();
        }
      };
    });

    Ok(Response::new(ReceiverStream::new(rx)))
  }

  async fn list(
    &self,
    request: tonic::Request<crate::rpc::ListRequest>,
  ) -> Result<tonic::Response<crate::rpc::ListResponse>, tonic::Status> {
    todo!()
  }

  async fn stats(
    &self,
    request: tonic::Request<crate::rpc::StatsRequest>,
  ) -> Result<tonic::Response<crate::rpc::StatsResponse>, tonic::Status> {
    todo!()
  }
}
