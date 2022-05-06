#[cfg(feature = "grpc")]
mod grpc;
#[cfg(feature = "lattice")]
mod lattice;

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::signal;
use tokio::sync::mpsc::Sender;
use vino_rpc::SharedRpcHandler;

use crate::options::Options;
pub(crate) type Result<T> = std::result::Result<T, crate::error::CliError>;

#[cfg(feature = "lattice")]
use vino_lattice::Lattice;
#[cfg(not(feature = "lattice"))]
#[derive(Debug, Copy, Clone)]
pub struct Lattice(); // Dummy struct if "lattice" feature is not enabled

#[cfg(feature = "reflection")]
pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("../../vino-rpc/src/generated/descriptors.bin");

#[derive(Debug)]
#[must_use]
/// Metadata for the running server.
pub struct ServerState {
  /// The address of the RPC server if it is running.
  pub rpc: Option<ServerControl>,

  /// True if we're connected to the lattice, false otherwise.
  pub lattice: Option<Arc<Lattice>>,
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
  pub fn stop_rpc_server(&self) {
    if let Some(ctl) = self.rpc.as_ref() {
      let _ = ctl.tx.send(ServerMessage::Close);
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

  if info.lattice.is_some() {
    something_started = true;
    info!("Host connected to lattice with id '{}'", info.id);
  }
  if !something_started {
    warn!("No server information available, did you intend to start a host without GRPC or a lattice connection?");
    warn!("If not, try passing the flag --rpc or --lattice to explicitly enable those features.");
  }
}

/// Starts an RPC server for the passed [vino_rpc::RpcHandler].
pub async fn start_server(provider: SharedRpcHandler, opts: Option<Options>) -> Result<ServerState> {
  debug!("Starting server with options: {:?}", opts);

  let opts = opts.unwrap_or_default();

  cfg_if::cfg_if! {
    if #[cfg(feature="grpc")] {
      let component_service = vino_invocation_server::InvocationServer::new(provider.clone());

      use vino_rpc::rpc::invocation_service_server::InvocationServiceServer;
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

  cfg_if::cfg_if! {
    if #[cfg(feature="lattice")] {
      let lattice = match &opts.lattice {
        Some(lattice) => {
          if lattice.enabled {
            let lattice =
              lattice::connect_to_lattice(lattice, opts.id.clone(), provider, opts.timeout).await?;
            Some(lattice)
          } else {
            None
          }
        }
        None => None,
      };
    } else {
      let lattice = None;
    }
  };

  Ok(ServerState {
    id: opts.id,
    rpc: ServerControl::maybe_new(rpc_addr),
    lattice,
  })
}

enum ServerMessage {
  Close,
}

/// Start a server with the passed [vino_rpc::RpcHandler] and keep it.
/// running until the process receives a SIGINT (^C).
pub async fn init_cli(provider: SharedRpcHandler, opts: Option<Options>) -> Result<()> {
  let state = start_server(provider, opts).await?;
  print_info(&state);

  info!("Waiting for ctrl-C");
  signal::ctrl_c().await?;
  println!(); // start on a new line.
  state.stop_rpc_server();

  Ok(())
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;
  use std::sync::Arc;
  use std::time::Duration;

  use anyhow::Result;
  use test_vino_provider::Provider;
  use tokio::time::sleep;
  use tonic::transport::Uri;
  use vino_invocation_server::connect_rpc_client;
  use vino_rpc::rpc::ListRequest;

  use super::*;
  use crate::options::ServerOptions;

  fn get_provider() -> SharedRpcHandler {
    Arc::new(Provider::default())
  }

  #[test_logger::test(tokio::test)]
  async fn test_starts() -> Result<()> {
    let mut options = Options::default();
    let rpc_opts = ServerOptions {
      enabled: true,
      ..Default::default()
    };
    options.rpc = Some(rpc_opts);
    let config = start_server(get_provider(), Some(options)).await?;
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
