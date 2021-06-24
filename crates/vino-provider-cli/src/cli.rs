use std::net::{
  IpAddr,
  Ipv4Addr,
  SocketAddr,
};
use std::str::FromStr;
use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::transport::Server;
use vino_rpc::rpc::invocation_service_server::InvocationServiceServer;
use vino_rpc::{
  InvocationServer,
  RpcHandler,
};

pub struct Options {
  pub port: u16,

  pub address: Ipv4Addr,
}

pub async fn init(
  provider: Arc<Mutex<dyn RpcHandler>>,
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

  let component_service = InvocationServer { provider };

  let svc = InvocationServiceServer::new(component_service);

  Server::builder().add_service(svc).serve(addr).await?;

  trace!("Server started");

  Ok(())
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;
  use std::time::Duration;

  use test_vino_provider::Provider;

  use super::*;
  use crate::Result;

  #[test_env_log::test(tokio::test)]
  async fn test_starts() -> Result<()> {
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
