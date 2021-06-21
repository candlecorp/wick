use std::sync::Arc;

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
use vino_guest::OutputPayload;
use vino_transport::serialize;

use crate::component_rpc_server::ComponentRpc;
use crate::{
  MessagePayload,
  Output,
  PayloadKind,
};
pub struct ComponentService {
  pub provider: Arc<Mutex<dyn crate::RpcHandler>>,
}

pub fn make_output(port: &str, inv_id: &str, payload: OutputPayload) -> Result<Output, Status> {
  match serialize(payload) {
    Ok(bytes) => Ok(Output {
      port: port.to_string(),
      invocation_id: inv_id.to_string(),
      payload: Some(MessagePayload {
        kind: PayloadKind::MessagePack.into(),
        value: bytes,
      }),
    }),
    Err(_) => Err(Status::failed_precondition("Could not serialize payload")),
  }
}

#[tonic::async_trait]
impl ComponentRpc for ComponentService {
  type InvokeStream = ReceiverStream<Result<Output, Status>>;

  async fn invoke(
    &self,
    request: tonic::Request<crate::Invocation>,
  ) -> Result<tonic::Response<Self::InvokeStream>, tonic::Status> {
    println!("ListFeatures = {:?}", request);

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
      if invocation.msg.is_none() {
        tx.send(Err(Status::failed_precondition("No payload received")))
          .await
          .unwrap();
        return;
      }
      let payload = invocation.msg.as_ref().unwrap().value.clone();
      debug!("Executing component {}", target.name.to_string());
      match &mut provider
        .request(invocation_id.to_string(), target.name.to_string(), payload)
        .await
      {
        Ok(receiver) => {
          while let Some((port_name, msg)) = receiver.next().await {
            debug!("got output {:?}", msg);
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

  async fn shutdown(
    &self,
    _request: tonic::Request<crate::ShutdownRequest>,
  ) -> Result<tonic::Response<crate::Ack>, tonic::Status> {
    panic!("Unimplemented");
  }
}
