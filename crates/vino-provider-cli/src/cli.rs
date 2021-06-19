use std::net::{
  IpAddr,
  Ipv4Addr,
  SocketAddr,
};
use std::str::FromStr;
use std::sync::Arc;

use futures::StreamExt;
use tokio::sync::{
  mpsc,
  Mutex,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;
use tonic::{
  Response,
  Status,
};
use vino_guest::OutputPayload;
use vino_provider::ProviderHandler;
use vino_rpc::component_rpc_server::{
  ComponentRpc,
  ComponentRpcServer,
};
use vino_rpc::{
  MessagePayload,
  Output,
  PayloadKind,
};
use vino_runtime::serialize;

pub struct ComponentService {
  pub provider: Arc<Mutex<dyn ProviderHandler>>,
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
    request: tonic::Request<vino_rpc::Invocation>,
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
    _request: tonic::Request<vino_rpc::ShutdownRequest>,
  ) -> Result<tonic::Response<vino_rpc::Ack>, tonic::Status> {
    panic!("Unimplemented");
  }
}

pub struct Options {
  pub port: u16,

  pub address: Ipv4Addr,
}

pub async fn init(
  provider: Arc<Mutex<dyn ProviderHandler>>,
  opts: Option<Options>,
) -> crate::Result<()> {
  let opts = match opts {
    Some(opts) => opts,
    None => Options {
      port: 54321,
      address: Ipv4Addr::from_str("127.0.0.1")?,
    },
  };

  let addr: SocketAddr = SocketAddr::new(IpAddr::V4(opts.address), opts.port);
  trace!("Binding to {:?}", addr.to_string());

  let component_service = ComponentService { provider };

  let svc = ComponentRpcServer::new(component_service);

  Server::builder().add_service(svc).serve(addr).await?;

  trace!("Server started");

  Ok(())
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;
  use std::time::Duration;

  use vino_provider_test::Provider;

  use super::*;
  use crate::Result;

  #[test_env_log::test(tokio::test)]
  async fn test() -> Result<()> {
    let init_handle = init(
      Arc::new(Mutex::new(Provider::default())),
      Some(Options {
        port: 12345,
        address: Ipv4Addr::from_str("127.0.0.1")?,
      }),
    );
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(1)) => {
            println!("timeout reached");
        }
        _ = init_handle => {
            panic!("server died");
        }
    };
    Ok(())
  }
}
