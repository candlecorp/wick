//! Vino RPC SDK

// !!START_LINTS
// Vino lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![deny(
    clippy::expect_used,
    clippy::explicit_deref_methods,
    clippy::option_if_let_else,
    clippy::await_holding_lock,
    clippy::cloned_instead_of_copied,
    clippy::explicit_into_iter_loop,
    clippy::flat_map_option,
    clippy::fn_params_excessive_bools,
    clippy::implicit_clone,
    clippy::inefficient_to_string,
    clippy::large_types_passed_by_value,
    clippy::manual_ok_or,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::must_use_candidate,
    clippy::needless_for_each,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::redundant_else,
    clippy::semicolon_if_nothing_returned,
    // clippy::too_many_lines,
    clippy::trivially_copy_pass_by_ref,
    clippy::unnested_or_patterns,
    clippy::future_not_send,
    clippy::useless_let_if_seq,
    clippy::str_to_string,
    clippy::inherent_to_string,
    clippy::let_and_return,
    clippy::string_to_string,
    clippy::try_err,
    clippy::if_then_some_else_none,
    bad_style,
    clashing_extern_declarations,
    const_err,
    // dead_code,
    deprecated,
    explicit_outlives_requirements,
    improper_ctypes,
    invalid_value,
    missing_copy_implementations,
    missing_debug_implementations,
    mutable_transmutes,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements ,
    patterns_in_fns_without_body,
    private_in_public,
    trivial_bounds,
    trivial_casts,
    trivial_numeric_casts,
    type_alias_bounds,
    unconditional_recursion,
    unreachable_pub,
    unsafe_code,
    unstable_features,
    // unused,
    unused_allocation,
    unused_comparisons,
    unused_import_braces,
    unused_parens,
    unused_qualifications,
    while_true,
    missing_docs
)]
// !!END_LINTS
// Add exceptions here
#![allow()]

use std::collections::HashMap;

use async_trait::async_trait;

/// Error module
pub mod error;

#[doc(hidden)]
pub mod generated;

/// Module for the [InvocationServer] implementation
pub mod invocation_server;

// TODO Need to get rid of or move this
#[doc(hidden)]
pub mod port;

/// Utility and conversion types
pub mod types;

/// Module with generated Tonic & Protobuf code
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

pub(crate) type Result<T> = std::result::Result<T, error::RpcError>;

/// The crate's error type
pub type Error = crate::error::RpcError;

/// The Result type for [RpcHandler] implementations
pub type RpcResult<T> = std::result::Result<T, Box<error::RpcError>>;

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate derivative;

/// A trait that implementers of the RPC interface should implement
#[async_trait]
pub trait RpcHandler: Send + Sync {
  /// Handle an incoming request for a target entity
  async fn request(
    &self,
    inv_id: String,
    entity: Entity,
    payload: HashMap<String, Vec<u8>>,
  ) -> RpcResult<BoxedPacketStream>;
  /// List the entities this [RpcHandler] manages
  async fn list_registered(&self) -> RpcResult<Vec<HostedType>>;
  /// Report the statists for all registered entities
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

#[doc(hidden)]
pub fn bind_new_socket() -> Result<tokio::net::TcpSocket> {
  let socket = tokio::net::TcpSocket::new_v4()?;
  socket.bind("127.0.0.1:0".parse().unwrap())?;
  Ok(socket)
}
