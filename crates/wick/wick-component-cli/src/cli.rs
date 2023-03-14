#[cfg(feature = "grpc")]
mod grpc;

use std::net::SocketAddr;

// use std::sync::Arc;
use tokio::signal;
use tokio::sync::mpsc::Sender;
use wick_rpc::SharedRpcHandler;

use crate::options::Options;
pub(crate) type Result<T> = std::result::Result<T, crate::error::CliError>;

#[cfg(not(feature = "mesh"))]
#[derive(Debug, Copy, Clone)]
pub struct Mesh(); // Dummy struct if "mesh" feature is not enabled

#[cfg(feature = "reflection")]
pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("../../wick-rpc/src/generated/descriptors.bin");

#[derive(Debug)]
#[must_use]
/// Metadata for the running server.
pub struct ServerState {
  /// The address of the RPC server if it is running.
  pub rpc: Option<ServerControl>,

  /// The ID of the server.
  pub id: String,
}

/// Struct that holds control methods and metadata for a running service.
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

  // if info.mesh.is_some() {
  //   something_started = true;
  //   info!("Host connected to mesh with id '{}'", info.id);
  // }
  if !something_started {
    warn!("No server information available, did you intend to start a host without GRPC or a mesh connection?");
    warn!("If not, try passing the flag --rpc or --mesh to explicitly enable those features.");
  }
}

/// Starts an RPC server for the passed [wick_rpc::RpcHandler].
pub async fn start_server(collection: SharedRpcHandler, opts: Option<Options>) -> Result<ServerState> {
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

  // cfg_if::cfg_if! {
  //   if #[cfg(feature="mesh")] {
  //     let mesh = match &opts.mesh {
  //       Some(mesh) => {
  //         if mesh.enabled {
  //           let mesh =
  //             mesh::connect_to_mesh(mesh, opts.id.clone(), collection, opts.timeout).await?;
  //           Some(mesh)
  //         } else {
  //           None
  //         }
  //       }
  //       None => None,
  //     };
  //   } else {
  // let mesh = None;
  // }
  // };

  Ok(ServerState {
    id: opts.id,
    rpc: ServerControl::maybe_new(rpc_addr),
    // mesh,
  })
}

enum ServerMessage {
  Close,
}

/// Start a server with the passed [wick_rpc::RpcHandler] and keep it.
/// running until the process receives a SIGINT (^C).
pub async fn init_cli(collection: SharedRpcHandler, opts: Option<Options>) -> Result<()> {
  let state = start_server(collection, opts).await?;
  print_info(&state);

  info!("Waiting for ctrl-C");
  signal::ctrl_c().await?;
  println!(); // start on a new line.
  state.stop_rpc_server().await;

  Ok(())
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;
  use std::sync::Arc;
  use std::time::Duration;

  use anyhow::Result;
  use test_native_collection::Collection;
  use tokio::time::sleep;
  use tonic::transport::Uri;
  use wick_invocation_server::connect_rpc_client;
  use wick_rpc::rpc::ListRequest;

  use super::*;
  use crate::options::ServerOptions;

  fn get_collection() -> SharedRpcHandler {
    Arc::new(Collection::default())
  }

  #[test_logger::test(tokio::test)]
  async fn test_starts() -> Result<()> {
    let mut options = Options::default();
    let rpc_opts = ServerOptions {
      enabled: true,
      ..Default::default()
    };
    options.rpc = Some(rpc_opts);
    let config = start_server(get_collection(), Some(options)).await?;
    let rpc = config.rpc.unwrap();
    debug!("Waiting for server to start");
    sleep(Duration::from_millis(100)).await;
    let uri = Uri::from_str(&format!("http://{}:{}", rpc.addr.ip(), rpc.addr.port())).unwrap();
    let mut client = connect_rpc_client(uri).await?;
    let response = client.list(ListRequest {}).await.unwrap();
    let list = response.into_inner();
    println!("list: {:?}", list);
    assert_eq!(list.schemas.len(), 1);
    Ok(())
  }
}
