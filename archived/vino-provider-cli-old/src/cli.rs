use std::net::{
  IpAddr,
  Ipv4Addr,
  SocketAddr,
};
use std::pin::Pin;
use std::sync::Arc;

use futures::{
  Future,
  StreamExt,
};
use log::*;
use structopt::StructOpt;
use tokio::net::TcpListener;
use tokio::sync::mpsc::channel;
use tokio::sync::Mutex;
use vino_macros::Ok;
use vino_provider::ProviderHandler;
use vino_rpc::peer::Peer;
use vino_rpc::rpc::{
  OutputMessage,
  VinoRpcMessage,
};

use crate::error::CliError;
use crate::Result;

#[derive(Debug, Clone, StructOpt)]
pub struct Options {
  /// Port to listen on
  #[structopt(short, long)]
  pub port: u16,

  /// IP address to bind to
  #[structopt(short, long, default_value = "127.0.0.1")]
  pub address: Ipv4Addr,
}

pub async fn init_basic(
  provider: impl Fn() -> Box<dyn VinoProvider> + Sync + Send + 'static,
  opts: Option<Options>,
) -> Result<()> {
  env_logger::init();
  let opts = match opts {
    Some(opts) => opts,
    None => Options::from_args(),
  };

  let addr: SocketAddr = SocketAddr::new(IpAddr::V4(opts.address), opts.port);
  trace!("Binding to {:?}", addr.to_string());
  let listener = TcpListener::bind(&addr).await?;
  trace!("Bound to {:?}", addr.to_string());

  tokio::spawn(async move {
    let provider = provider();
    let (tx, rx) = channel();
    loop {
      trace!("Waiting for connection");
      let socket = match listener.accept().await {
        Ok((socket, _)) => socket,
        Err(e) => {
          return Err(CliError::Other(format!("error on TcpListener: {}", e)));
        }
      };
      debug!(
        "Server accepting stream from: {}",
        socket.peer_addr().unwrap()
      );
      let provider = Arc::new(Mutex::new(provider));
      tokio::spawn(async move {
        debug!("Creating new peer to handle connections");
        let mut client = Peer::new("server".to_string(), socket);
        loop {
          let next = client.next().await?.unwrap();
          trace!("Server got {} msg", next.op_name());
          match next {
            VinoRpcMessage::Invoke(invocation) => {
              trace!("Got invocation with ID: {}", invocation.id);
              let component = invocation.target.into_provider()?;
              let payload = invocation.msg.into_bytes()?;
              let provider = provider.lock().await;

              let result = provider
                .request(invocation.id.to_string(), component, payload)
                .await;
              if let Err(err) = result {
                client.send(&VinoRpcMessage::Error(err.to_string())).await?;
              } else {
                let mut receiver = result.unwrap();
                trace!("waiting for output from request {}", invocation.id);
                while let Some((port_name, msg)) = receiver.next().await {
                  debug!("got output {:?}", msg);
                  client
                    .send(&VinoRpcMessage::Output(OutputMessage {
                      invocation_id: invocation.id.to_string(),
                      port: port_name.to_string(),
                      payload: msg.into(),
                    }))
                    .await?
                }
              }
            }
            VinoRpcMessage::Shutdown => {
              warn!("Shutting down");
              break;
            }
            _ => panic!("Message not handled"),
          }
        }
        warn!("Shut down");
        Ok!(())
      });
    }
    Ok!(())
  })
  .await??;
  Ok(())
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use vino_provider_test::Provider;

  use super::*;

  fn factory() -> Box<dyn VinoProvider> {
    Box::new(Provider::default())
  }

  #[test_env_log::test(tokio::test)]
  async fn test() -> Result<()> {
    // let factory = Box::pin(factory);
    let init_handle = init_basic(
      factory,
      Some(Options {
        port: 12345,
        address: Ipv4Addr::from_str("127.0.0.1")?,
      }),
    );
    Ok(())
  }
}
