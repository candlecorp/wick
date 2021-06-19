use futures::lock::Mutex;
use rmp_futures::rpc::decode::RpcMessage;
use rmp_futures::rpc::decode::RpcParamsFuture;
use rmp_futures::rpc::decode::RpcStream;
use rmp_futures::rpc::encode::RpcSink;
use rmp_futures::rpc::{MsgId, RequestDispatch};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncRead;
use tokio::join;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::TcpStream;
use tokio::time::sleep;

async fn hello_handler<R>(id: MsgId, params: RpcParamsFuture<R>) -> R
where
  R: AsyncRead + Unpin,
{
  let params = params.params().await.unwrap();
  let (param1, reader) = params
    .last()
    .unwrap()
    .decode()
    .await
    .unwrap()
    .into_string()
    .unwrap()
    .into_string()
    .await
    .unwrap();
  tokio::spawn(async move {
    println!("got hello with id={:?} param={}", id, param1);
  });
  reader
}

async fn handler() -> io::Result<()> {
  let stream = TcpStream::connect("127.0.0.1:12345").await?;
  let (reader, writer) = stream.into_split();
  let w = Arc::new(Mutex::new(Some(writer)));
  let mut reader = RpcStream::new(reader);
  let inbound = tokio::spawn(async move {
    loop {
      println!("INBOUND::[waiting...]");
      reader = match reader.next().await? {
        RpcMessage::Request(req) => {
          println!("<<< Got inbound request for {:?}", req);
          let id = req.id();
          let method = req.method().await?;
          let (method, params) = method.into_string().await?;
          println!("<<< Method: {}", method);
          match method.as_ref() {
            "hello" => hello_handler(id, params).await,
            _ => panic!("unknown method"),
          }
        }
        RpcMessage::Response(resp) => {
          println!("<<< Got inbound response for {:?}", resp);
          resp.skip().await?
        }
        RpcMessage::Notify(_nfy) => panic!("got notify"),
      };
    }
    #[allow(unreachable_code)]
    Ok::<(), std::io::Error>(())
  });
  let outbound = tokio::spawn(async move {
    let dispatch: Arc<RequestDispatch<OwnedReadHalf>> = Arc::new(RequestDispatch::default());

    let writer = w.lock().await.take().unwrap();
    let mut sink = RpcSink::new(writer);
    for num in 0..10 {
      println!(">>> {} Outbound sleeping for {} seconds", num, 1);
      sleep(Duration::from_secs(1)).await;

      println!(">>> {} Outbound writing request to socket", num,);
      let (args, _reply) = dispatch.write_request(sink, "hello", 1).await;
      sink = args?.last().write_str(&format!("Num is {}", num)).await?;
      println!(">>> {} Outbound done writing, moving on", num,);
    }
    #[allow(unreachable_code)]
    Ok::<(), std::io::Error>(())
  });
  let (_, _) = join!(inbound, outbound);
  Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
  env_logger::init();
  println!("starting client");
  match handler().await {
    Ok(_) => println!("it was ok!"),
    Err(e) => eprintln!("got error: {:?}", e),
  }
  println!("closing client");
  Ok(())
}
