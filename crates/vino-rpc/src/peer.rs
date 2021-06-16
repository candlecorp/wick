use futures::lock::Mutex;
use log::error;
use rmp_futures::rpc::decode::{RpcMessage, RpcStream};
use rmp_futures::rpc::encode::RpcSink;
use rmp_futures::rpc::{MsgId, RequestDispatch};
use vino_runtime::serialize;

use std::sync::Arc;

use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;

use tokio::net::TcpStream;

use crate::handlers;
use crate::rpc::{self, VinoRpcMessage};

use crate::{Error, Result};

pub type RpcResult = Result<RpcMessage<RpcStream<OwnedReadHalf>>>;
#[derive(Debug)]
pub struct Peer {
    pub id: String,
    pub reader: Option<RpcStream<OwnedReadHalf>>,
    pub writer: Arc<Mutex<Option<OwnedWriteHalf>>>,
    pub shutting_down: bool,
}

impl Peer {
    pub fn new(id: String, stream: TcpStream) -> Self {
        let (reader, writer) = stream.into_split();
        let reader = RpcStream::new(reader);

        Self {
            id,
            reader: Some(reader),
            writer: Arc::new(Mutex::new(Some(writer))),
            shutting_down: false,
        }
    }
    // #[instrument]
    pub async fn send(&self, msg: &VinoRpcMessage) -> Result<()> {
        debug!("[{}] sending msg", self.id);
        let mut writer_option = self.writer.lock().await;
        let writer = writer_option.take().unwrap();
        let dispatch: RequestDispatch<OwnedReadHalf> = RequestDispatch::default();

        let sink = RpcSink::new(writer);
        let operation = msg.op_name();
        let value = serialize(msg)?;
        let (args, _reply) = dispatch.write_request(sink, operation, 1).await;
        let sink = args?.last().write_str_bytes(&value).await?;
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
        let value = serialize(&VinoRpcMessage::Ack(id.to_string()))?;
        let writer = sink
            .write_ok_response(id, |rsp| rsp.write_str_bytes(&value))
            .await
            .unwrap()
            .into_inner();
        writer_option.replace(writer);
        Ok(())
    }
    // #[instrument]
    pub async fn next(&mut self) -> Result<Option<VinoRpcMessage>> {
        debug!("[{}] waiting for next message...", self.id);
        let mut reader = self.reader.take().unwrap();
        if self.shutting_down {
            return Err(Error::ShuttingDown);
        }
        let (message, reader) = loop {
            let (m, r) = match reader.next().await? {
                RpcMessage::Request(req) => {
                    let id = req.id();
                    debug!("[{}] got request ID:{}", self.id, id);
                    let method = req.method().await?;
                    let (method, params) = method.into_string().await?;
                    debug!("[{}] request method parsed to '{}'", self.id, method);
                    let (message, reader) = match method.as_ref() {
                        rpc::OP_INVOKE => handlers::handler(id, params).await?,
                        rpc::OP_ERROR => handlers::handler(id, params).await?,
                        rpc::OP_CLOSE => handlers::handler(id, params).await?,
                        rpc::OP_OUTPUT => handlers::handler(id, params).await?,
                        rpc::OP_ACK => {
                            error!("Ack request received. Ack only makes sense as a response.");
                            (None, params.params().await?.skip().await?)
                        }
                        rpc::OP_PING => {
                            trace!("<PING>");
                            self.send(&VinoRpcMessage::Pong).await?;
                            (None, params.params().await?.skip().await?)
                        }
                        rpc::OP_PONG => {
                            trace!("<PONG>");
                            (None, params.params().await?.skip().await?)
                        }
                        rpc::OP_SHUTDOWN => {
                            self.shutting_down = true;
                            (None, params.params().await?.skip().await?)
                        }
                        _ => panic!("unhandled method {}", method),
                    };
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
    use crate::rpc::{ClosePayload, OutputPayload};
    use tokio::net::TcpListener;
    use vino_runtime::{Invocation, MessagePayload, VinoEntity};

    use super::*;

    async fn make_server() -> tokio::task::JoinHandle<Result<()>> {
        warn!("Starting server");

        let listener = TcpListener::bind("127.0.0.1:12345").await.unwrap();
        tokio::spawn(async move {
            // loop {
            let socket = match listener.accept().await {
                Ok((socket, _)) => socket,
                Err(e) => {
                    return Err(Error::Other(format!("error on TcpListener: {}", e)));
                }
            };
            tokio::spawn(async move {
                warn!(
                    "Server accepting stream from: {}",
                    socket.peer_addr().unwrap()
                );
                let mut peer = Peer::new("server".to_string(), socket);
                loop {
                    let next = peer.next().await?.unwrap();
                    warn!("Server got {} msg", next.op_name());
                    match next {
                        VinoRpcMessage::Invoke(invocation) => {
                            warn!("invoke: {}", invocation.id);
                            assert_eq!(invocation.id, "INV_ID");
                            peer.send(&VinoRpcMessage::Output(OutputPayload {
                                tx_id: invocation.tx_id,
                                ..OutputPayload::default()
                            }))
                            .await?
                        }
                        VinoRpcMessage::Output(output) => {
                            warn!("output.tx_id: {}", output.tx_id);
                            assert_eq!(output.tx_id, "TX_ID");
                            peer.send(&VinoRpcMessage::Close(ClosePayload {
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
                        VinoRpcMessage::Ack(id) => {
                            warn!("ack: {}", id);
                        }
                        VinoRpcMessage::Ping => {
                            warn!("Server got ping");
                        }
                        VinoRpcMessage::Pong => {
                            warn!("Server got pong");
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
            // }
            Ok!(())
        })
    }

    #[test_env_log::test]
    fn test_invoke() -> anyhow::Result<()> {
        trace!("test invoke");

        fn test() -> tokio::task::JoinHandle<Result<()>> {
            tokio::task::spawn(async move {
                let stream = TcpStream::connect("127.0.0.1:12345").await?;
                info!("Connected to server");
                let mut peer = Peer::new("client".to_string(), stream);
                let invoke = VinoRpcMessage::Invoke(Invocation {
                    origin: VinoEntity::Component("".to_string()),
                    target: VinoEntity::Component("".to_string()),
                    operation: "".to_string(),
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
                Ok!(())
            })
        }
        let (res1, res2) = tokio::runtime::Runtime::new()?
            .block_on(async move { tokio::join!(make_server().await, test()) });
        trace!("Futures complete");
        let _server_response = res1?;
        let _test_response = res2?;

        Ok(())
    }

    #[test_env_log::test]
    fn test_output() -> anyhow::Result<()> {
        trace!("test output");

        fn test() -> tokio::task::JoinHandle<Result<()>> {
            tokio::task::spawn(async move {
                let stream = TcpStream::connect("127.0.0.1:12345").await?;
                trace!("Connected to server");
                let mut peer = Peer::new("client".to_string(), stream);
                let msg = VinoRpcMessage::Output(OutputPayload {
                    tx_id: "TX_ID".to_string(),
                    ..Default::default()
                });
                peer.send(&msg).await?;
                info!("Sent");
                let next = peer.next().await?;
                trace!("Next was : {:?}", next);
                peer.send_shutdown().await?;
                Ok!(())
            })
        }
        let (res1, res2) = tokio::runtime::Runtime::new()?
            .block_on(async move { tokio::join!(make_server().await, test()) });

        trace!("Futures complete");
        let _server_response = res1?;
        let _test_response = res2?;

        Ok(())
    }
}
