use futures::executor::LocalPool;
use futures::task::LocalSpawnExt;
use std::io;
use std::sync::Arc;
use tokio::io::AsyncRead;
use tokio::net::TcpStream;

use rmp_futures::rpc::decode::RpcStream;
use rmp_futures::rpc::encode::RpcSink;
use rmp_futures::rpc::RequestDispatch;
use rmp_futures::rpc::ResponseReceiver;

#[tokio::main]
async fn main() -> io::Result<()> {
  let mut pool = LocalPool::new();
  let spawner = pool.spawner();
  pool.run_until(async {
    let socket = TcpStream::connect("127.0.0.1:12345").await?;

    let (reader, writer) = socket.into_split();
    let sink = RpcSink::new(writer);
    let stream = RpcStream::new(reader);
    let dispatch = Arc::new(RequestDispatch::default());
    let dispatch2 = dispatch.clone();

    spawner
      .spawn_local(async move {
        dispatch2.dispatch(stream).await.unwrap();
      })
      .unwrap();

    let (args, reply1) = dispatch.write_request(sink, "hello", 1).await;
    let sink = args?.last().write_str("Joy").await?;

    let (args, reply2) = dispatch.write_request(sink, "hello", 1).await;
    let _sink = args?.last().write_str("Sugar").await?;

    async fn print_reply(reply: ResponseReceiver<impl AsyncRead + Unpin>) {
      match reply.await.unwrap() {
        Ok(vf) => {
          let (s, _stream) = vf.into_string().unwrap().into_string().await.unwrap();
          println!("got good response s={:?}", s);
        }
        Err(vf) => {
          let (s, _stream) = vf.into_string().unwrap().into_string().await.unwrap();
          println!("got bad response s={:?}", s);
        }
      }
    }

    futures::join!(print_reply(reply1), print_reply(reply2));

    Ok(())
  })
}
