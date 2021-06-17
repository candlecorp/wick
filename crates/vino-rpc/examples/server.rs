use log::*;
use tokio::net::TcpListener;
use vino_macros::Ok;
use vino_rpc::peer::Peer;
use vino_rpc::rpc::{
  CloseMessage,
  OutputMessage,
  VinoRpcMessage,
};
use vino_rpc::{
  Error,
  Result,
};

#[tokio::main]
async fn main() -> Result<()> {
  env_logger::init();
  info!("Starting server");

  let listener = TcpListener::bind("127.0.0.1:12345").await.unwrap();
  tokio::spawn(async move {
    loop {
      trace!("Waiting for connection");
      let socket = match listener.accept().await {
        Ok((socket, _)) => socket,
        Err(e) => {
          return Err(Error::Other(format!("error on TcpListener: {}", e)));
        }
      };
      debug!(
        "Server accepting stream from: {}",
        socket.peer_addr().unwrap()
      );
      tokio::spawn(async move {
        debug!("Creating new peer to handle connections");
        let mut peer = Peer::new("server".to_string(), socket);
        loop {
          let next = peer.next().await?.unwrap();
          trace!("Server got {} msg", next.op_name());
          match next {
            VinoRpcMessage::Invoke(invocation) => {
              trace!("invoke: {}", invocation.id);
              assert_eq!(invocation.id, "INV_ID");
              peer
                .send(&VinoRpcMessage::Output(OutputMessage {
                  tx_id: invocation.tx_id,
                  ..OutputMessage::default()
                }))
                .await?
            }
            VinoRpcMessage::Output(output) => {
              trace!("output.tx_id: {}", output.tx_id);
              assert_eq!(output.tx_id, "TX_ID");
              peer
                .send(&VinoRpcMessage::Close(CloseMessage {
                  tx_id: output.tx_id,
                  entity: output.entity,
                }))
                .await?
            }
            VinoRpcMessage::Close(close) => {
              trace!("close.tx_id: {}", close.tx_id);
              assert_eq!(close.tx_id, "TX_ID");
            }
            VinoRpcMessage::Error(err) => {
              trace!("err: {}", err);
              assert_eq!(err, "ERROR");
            }
            VinoRpcMessage::Ping(s) => {
              warn!("Server got ping: {}", s);
            }
            VinoRpcMessage::Pong(s) => {
              warn!("Server got pong: {}", s);
            }
            VinoRpcMessage::Shutdown => {
              warn!("Shutting down");
              break;
            }
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
