//! Vino RPC implementation

#![deny(
  warnings,
  missing_debug_implementations,
  trivial_casts,
  trivial_numeric_casts,
  unsafe_code,
  unstable_features,
  unused_import_braces,
  unused_qualifications,
  type_alias_bounds,
  trivial_bounds,
  mutable_transmutes,
  invalid_value,
  explicit_outlives_requirements,
  deprecated,
  clashing_extern_declarations,
  clippy::expect_used,
  clippy::explicit_deref_methods,
  // missing_docs
)]
#![warn(clippy::cognitive_complexity)]

use std::collections::HashMap;

use async_trait::async_trait;
pub mod error;
pub mod generated;
pub mod invocation_server;
pub mod port;
pub mod types;
pub use generated::vino as rpc;
use generated::vino::invocation_service_client::InvocationServiceClient;
use generated::vino::invocation_service_server::InvocationServiceServer;
pub use invocation_server::InvocationServer;
use tokio::task::JoinHandle;
use tonic::transport::{
  Channel,
  Server,
  Uri,
};
pub use types::*;
use vino_entity::Entity;
pub type Result<T> = std::result::Result<T, error::RpcError>;
pub type Error = crate::error::RpcError;
pub type RpcResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate derivative;

#[async_trait]
pub trait RpcHandler: Send + Sync {
  async fn request(
    &self,
    inv_id: String,
    entity: Entity,
    payload: HashMap<String, Vec<u8>>,
  ) -> RpcResult<BoxedPacketStream>;
  async fn list_registered(&self) -> RpcResult<Vec<HostedType>>;
  async fn report_statistics(&self, id: Option<String>) -> RpcResult<Vec<Statistics>>;
}

/// Build and spawn an RPC server for the passed provider
pub fn make_rpc_server(
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

/// Create an RPC client
pub async fn make_rpc_client(uri: Uri) -> Result<InvocationServiceClient<Channel>> {
  Ok(InvocationServiceClient::connect(uri).await?)
}

pub fn bind_new_socket() -> Result<tokio::net::TcpSocket> {
  let socket = tokio::net::TcpSocket::new_v4()?;
  socket.bind("127.0.0.1:0".parse().unwrap())?;
  Ok(socket)
}
