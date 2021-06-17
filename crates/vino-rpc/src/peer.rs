use std::sync::Arc;

use futures::lock::Mutex;
use rmp_futures::rpc::decode::{
  RpcMessage,
  RpcStream,
};
use rmp_futures::rpc::encode::RpcSink;
use rmp_futures::rpc::{
  MsgId,
  RequestDispatch,
};
use tokio::net::tcp::{
  OwnedReadHalf,
  OwnedWriteHalf,
};
use tokio::net::TcpStream;
use vino_runtime::serialize;

use crate::handlers::{self,};
use crate::rpc::VinoRpcMessage;
use crate::Result;

pub type RpcResult = Result<RpcMessage<RpcStream<OwnedReadHalf>>>;
#[derive(Debug)]
pub struct Peer {
  pub id: String,
  pub reader: Option<RpcStream<OwnedReadHalf>>,
  pub writer: Arc<Mutex<Option<OwnedWriteHalf>>>,
}

impl Peer {
  pub fn new(id: String, stream: TcpStream) -> Self {
    let (reader, writer) = stream.into_split();
    let reader = RpcStream::new(reader);

    Self {
      id,
      reader: Some(reader),
      writer: Arc::new(Mutex::new(Some(writer))),
    }
  }
  // #[instrument]
  pub async fn send(&self, msg: &VinoRpcMessage) -> Result<()> {
    debug!("[{}] sending [{}] message", self.id, msg.op_name());
    let operation = msg.op_name();
    let value = serialize(msg)?;
    self.send_raw(operation, &value).await
  }
  async fn send_raw(&self, method: &str, msg: &[u8]) -> Result<()> {
    debug!("[{}] sending [{}] {} bytes", self.id, method, msg.len());
    let mut writer_option = self.writer.lock().await;
    let writer = writer_option.take().unwrap();
    let dispatch: RequestDispatch<OwnedReadHalf> = RequestDispatch::default();
    let sink = RpcSink::new(writer);
    let (args, _reply) = dispatch.write_request(sink, method, 1).await;
    let sink = args?.last().write_str_bytes(msg).await?;
    writer_option.replace(sink.into_inner());
    Ok(())
  }
  // #[instrument]
  pub async fn send_shutdown(&self) -> Result<()> {
    debug!("[{}] sending shutdown", self.id);
    let msg = VinoRpcMessage::Shutdown;
    self.send(&msg).await
  }
  // #[instrument]
  pub async fn send_response(&self, id: MsgId) -> Result<()> {
    debug!("[{}] sending ack for ID:{}", self.id, id);
    let mut writer_option = self.writer.lock().await;
    let writer = writer_option.take().unwrap();
    let sink = RpcSink::new(writer);
    let writer = sink
      .write_ok_response(id, |rsp| rsp.write_bool(true))
      .await?;
    let writer = writer.into_inner();
    writer_option.replace(writer);
    Ok(())
  }
  // #[instrument]
  pub async fn next(&mut self) -> Result<Option<VinoRpcMessage>> {
    debug!("[{}] waiting for next message...", self.id);
    let mut reader = self.reader.take().unwrap();

    let (message, reader) = loop {
      let (m, r) = match reader.next().await? {
        RpcMessage::Request(req) => {
          let id = req.id();
          debug!("[{}] got request ID:{}", self.id, id);
          let method = req.method().await?;
          let (method, params) = method.into_string().await?;
          debug!("[{}] request method parsed to '{}'", self.id, method);
          let (message, reader) = handlers::handle(&method, id, params).await?;
          self.send_response(id).await?;
          (message, reader)
        }
        RpcMessage::Response(resp) => {
          debug!("[{}] got response for ID:{}", self.id, resp.id());
          (None, resp.skip().await?)
        }
        RpcMessage::Notify(_nfy) => panic!("got notify"),
      };
      reader = r;
      if m.is_some() {
        break (m, reader);
      }
    };

    debug!("[{}] finished processing message", self.id);
    self.reader.replace(reader);

    Ok(message)
  }
}

#[cfg(test)]
mod tests {

  use tokio::net::TcpListener;
  use vino_runtime::{
    Invocation,
    MessagePayload,
    VinoEntity,
  };

  use super::*;
  use crate::rpc::{
    CloseMessage,
    OutputMessage,
  };
  use crate::Error;

  async fn make_server(port: &'static str) -> tokio::task::JoinHandle<Result<()>> {
    warn!("Starting server");

    let handle = tokio::spawn(async move {
      let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
      let socket = match listener.accept().await {
        Ok((socket, _)) => socket,
        Err(e) => {
          return Err(Error::Other(format!("error on TcpListener: {}", e)));
        }
      };
      tokio::spawn(async move {
        warn!("Server accepting stream from: {}", socket.peer_addr()?);
        let mut peer = Peer::new("server".to_string(), socket);
        loop {
          let next = peer.next().await?.unwrap();
          warn!("Server got {} msg", next.op_name());
          match next {
            VinoRpcMessage::Invoke(invocation) => {
              warn!("invoke: {}", invocation.id);
              assert_eq!(invocation.id, "INV_ID");
              peer
                .send(&VinoRpcMessage::Output(OutputMessage {
                  tx_id: invocation.tx_id,
                  ..OutputMessage::default()
                }))
                .await?
            }
            VinoRpcMessage::Output(output) => {
              warn!("output.tx_id: {}", output.tx_id);
              assert_eq!(output.tx_id, "TX_ID");
              peer
                .send(&VinoRpcMessage::Close(CloseMessage {
                  tx_id: output.tx_id,
                  entity: output.entity,
                }))
                .await?
            }
            VinoRpcMessage::Close(close) => {
              warn!("close.tx_id: {}", close.tx_id);
              assert_eq!(close.tx_id, "TX_ID");
            }
            VinoRpcMessage::Error(err) => {
              warn!("err: {}", err);
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
        warn!("Shutting down");
        Ok!(())
      });
      warn!("Connection running");
      Ok!(())
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    handle
  }

  #[test_env_log::test(tokio::test)]
  async fn test_invoke() -> Result<()> {
    trace!("test invoke");
    let port = "12345";
    let server = make_server(port).await;
    let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
    info!("Connected to server");
    let mut peer = Peer::new("client".to_string(), stream);
    let invoke = VinoRpcMessage::Invoke(Invocation {
      origin: VinoEntity::Component("".to_string()),
      target: VinoEntity::Component("".to_string()),
      msg: MessagePayload::MessagePack(vec![]),
      id: "INV_ID".to_string(),
      tx_id: "TX_ID".to_string(),
      encoded_claims: "".to_string(),
      host_id: "".to_string(),
    });
    info!("Sending invocation");
    peer.send(&invoke).await?;
    info!("Sent");
    let next = peer.next().await?.unwrap();
    info!("Next was : {:?}", next);
    if let VinoRpcMessage::Output(output) = next {
      assert_eq!(output.tx_id, "TX_ID");
    } else {
      panic!("wrong message returned");
    }
    peer.send_shutdown().await?;
    server.await??;

    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn test_output() -> Result<()> {
    trace!("test output");
    let port = "12346";
    let server = make_server(port).await;
    let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;

    trace!("Connected to server");
    let mut peer = Peer::new("client".to_string(), stream);
    let msg = VinoRpcMessage::Output(OutputMessage {
      tx_id: "TX_ID".to_string(),
      ..Default::default()
    });
    peer.send(&msg).await?;
    info!("Sent");
    let next = peer.next().await?;
    trace!("Next was : {:?}", next);
    peer.send_shutdown().await?;
    server.await??;

    Ok(())
  }
}
