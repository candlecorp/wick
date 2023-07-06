#[cfg(feature = "grpc")]
mod grpc;

use std::net::SocketAddr;

use flow_component::SharedComponent;
// use std::sync::Arc;
use tokio::signal;
use tokio::sync::mpsc::Sender;
use tracing::{debug, info, warn};

use crate::options::Options;
pub(crate) type Result<T> = std::result::Result<T, crate::error::CliError>;

#[cfg(feature = "reflection")]
pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("../../wick-rpc/src/generated/descriptors.bin");

#[derive(Debug, Clone)]
#[must_use]
/// Metadata for the running server.
pub struct ServerState {
  /// The address of the RPC server if it is running.
  pub rpc: Option<ServerControl>,

  /// The ID of the server.
  pub id: String,
}

/// Struct that holds control methods and metadata for a running service.
#[derive(Clone)]
pub struct ServerControl {
  /// The address of the RPC server.
  pub addr: SocketAddr,
  tx: Sender<ServerMessage>,
}

impl std::fmt::Debug for ServerControl {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ServerControl").field("addr", &self.addr).finish()
  }
}

impl ServerControl {
  fn maybe_new(opt: Option<(SocketAddr, Sender<ServerMessage>)>) -> Option<Self> {
    if let Some((addr, tx)) = opt {
      Some(Self { addr, tx })
    } else {
      None
    }
  }
}

impl ServerState {
  /// Stop the RPC server if it's running.
  pub async fn stop_rpc_server(&self) {
    if let Some(ctl) = self.rpc.as_ref() {
      let _ = ctl.tx.send(ServerMessage::Close).await;
    }
  }
}

#[doc(hidden)]
pub fn print_info(info: &ServerState) {
  let mut something_started = false;
  if let Some(addr) = &info.rpc {
    let addr = addr.addr;
    something_started = true;
    info!("GRPC server bound to {} on port {}", addr.ip(), addr.port());
  }

  if !something_started {
    warn!("No server information available, did you intend to start a host without GRPC or a mesh connection?");
    warn!("If not, try passing the flag --rpc or --mesh to explicitly enable those features.");
  }
}

/// Starts an RPC server for the passed [SharedComponent].
pub async fn start_server(collection: SharedComponent, opts: Option<Options>) -> Result<ServerState> {
  debug!("Starting server with options: {:?}", opts);

  let opts = opts.unwrap_or_default();

  cfg_if::cfg_if! {
    if #[cfg(feature="grpc")] {
      let component_service = wick_invocation_server::InvocationServer::new(collection.clone());

      use wick_rpc::rpc::invocation_service_server::InvocationServiceServer;
      let svc = InvocationServiceServer::new(component_service);

      let rpc_addr = if let Some(rpc_options) = &opts.rpc {
        if !rpc_options.enabled {
          None
        } else {
          let addr = grpc::start_rpc_server(rpc_options, svc.clone()).await?;
          Some(addr)
        }
      } else {
        None
      };
    } else {
      let rpc_addr = None;
    }
  };

  Ok(ServerState {
    id: opts.id,
    rpc: ServerControl::maybe_new(rpc_addr),
  })
}

enum ServerMessage {
  Close,
}

/// Start a server with the passed [SharedComponent] and keep it
/// running until the process receives a SIGINT (^C).
pub async fn init_cli(collection: SharedComponent, opts: Option<Options>) -> Result<()> {
  let state = start_server(collection, opts).await?;
  print_info(&state);

  info!("Waiting for ctrl-C");
  signal::ctrl_c().await?;
  println!(); // start on a new line.
  state.stop_rpc_server().await;

  Ok(())
}
