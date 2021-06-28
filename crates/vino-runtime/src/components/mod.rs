use std::path::Path;

pub mod grpc_url_provider;
pub mod native_provider;
// pub mod provider_component;
pub mod vino_component;
pub(crate) mod wapc_component_actor;

pub(crate) type Inputs = Vec<String>;
pub(crate) type Outputs = Vec<String>;

use actix::prelude::*;
use actix_rt::task::JoinHandle;
use tonic::transport::Server;
use vino_rpc::rpc::invocation_service_server::InvocationServiceServer;
use vino_rpc::{
  HostedType,
  InvocationServer,
  RpcHandler,
  Statistics,
};

use self::vino_component::WapcComponent;
use crate::{
  Invocation,
  Result,
};

pub fn load_wasm_from_file(path: &Path) -> Result<WapcComponent> {
  WapcComponent::from_file(path)
}

pub async fn load_wasm_from_oci(
  actor_ref: &str,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
) -> Result<WapcComponent> {
  let actor_bytes =
    crate::util::oci::fetch_oci_bytes(actor_ref, allow_latest, &allowed_insecure).await?;
  Ok(WapcComponent::from_slice(&actor_bytes)?)
}

pub async fn load_wasm(
  actor_ref: &str,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
) -> Result<WapcComponent> {
  let path = Path::new(&actor_ref);
  if path.exists() {
    Ok(WapcComponent::from_file(path)?)
  } else {
    load_wasm_from_oci(actor_ref, allow_latest, allowed_insecure).await
  }
}

#[derive(Debug)]
pub(crate) enum ProviderMessage {
  Invoke(Invocation),
  List(ListRequest),
  Statistics(StatsRequest),
}

impl Message for ProviderMessage {
  type Result = Result<ProviderResponse>;
}

#[derive(Debug)]
pub(crate) struct ListRequest {}
#[derive(Debug)]
pub(crate) struct StatsRequest {}

#[derive(Debug)]
pub(crate) enum ProviderResponse {
  InvocationResponse,
  List(Vec<HostedType>),
  Stats(Vec<Statistics>),
}

impl ProviderResponse {
  pub(crate) fn into_list_response(self) -> Result<Vec<HostedType>> {
    match self {
      ProviderResponse::List(v) => Ok(v),
      _ => Err(crate::error::VinoError::ConversionError(
        "into_list_response",
      )),
    }
  }
  pub(crate) fn into_stats_response(self) -> Result<Vec<Statistics>> {
    match self {
      ProviderResponse::Stats(v) => Ok(v),
      _ => Err(crate::error::VinoError::ConversionError(
        "into_stats_response",
      )),
    }
  }
  pub(crate) fn into_invocation_response(self) -> Result<()> {
    todo!()
  }
}

pub(crate) fn make_grpc_server(
  socket: tokio::net::TcpSocket,
  provider: impl RpcHandler + 'static,
) -> JoinHandle<std::result::Result<(), tonic::transport::Error>> {
  let component_service = InvocationServer::new(provider);

  let svc = InvocationServiceServer::new(component_service);

  let listener = tokio_stream::wrappers::TcpListenerStream::new(socket.listen(1).unwrap());

  tokio::spawn(
    Server::builder()
      .add_service(svc)
      .serve_with_incoming(listener),
  )
}

pub(crate) fn bind_new_socket() -> Result<tokio::net::TcpSocket> {
  let socket = tokio::net::TcpSocket::new_v4()?;
  socket.bind("127.0.0.1:0".parse().unwrap())?;
  Ok(socket)
}
