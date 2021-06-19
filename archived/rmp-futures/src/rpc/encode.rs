use std::pin::Pin;
use std::task::{Context, Poll};

use futures::io::Result as IoResult;
use futures::Future;
use tokio::io::AsyncWrite;

use super::*;
use crate::encode::{ArrayFuture, MsgPackWriter};

pub struct RpcSink<W> {
  writer: W,
}

impl<W: AsyncWrite + Unpin> AsyncWrite for RpcSink<W> {
  fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<IoResult<usize>> {
    W::poll_write(Pin::new(&mut self.as_mut().writer), cx, buf)
  }

  fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<IoResult<()>> {
    W::poll_flush(Pin::new(&mut self.as_mut().writer), cx)
  }

  fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<IoResult<()>> {
    W::poll_shutdown(Pin::new(&mut self.as_mut().writer), cx)
  }
}

impl<W: AsyncWrite + Unpin> RpcSink<W> {
  pub fn new(writer: W) -> Self {
    RpcSink { writer }
  }

  pub fn into_inner(self) -> W {
    self.writer
  }

  #[must_use = "dropping the writer may leave the message unfinished"]
  pub async fn write_request(
    self,
    msgid: MsgId,
    method: impl AsRef<str>,
    num_args: u32,
  ) -> IoResult<ArrayFuture<RpcSink<W>>> {
    // First, wrap our RpcSink in a MsgPackWriter rather than using the
    // underlying writer. When this message is fully written and its writer
    // is returned, the client will be left with this RpcSink pointing at
    // the next message.
    let args = MsgPackWriter::new(self)
      .write_array_len(4)
      .await?
      .next()
      .write_int(MsgType::Request)
      .await?
      .next()
      .write_int(msgid)
      .await?
      .next()
      .write_str(method.as_ref())
      .await?
      .last();
    args.write_array_len(num_args).await
  }

  pub async fn write_ok_response<F, Fut>(self, msgid: MsgId, write_ok: F) -> IoResult<RpcSink<W>>
  where
    F: FnOnce(MsgPackWriter<RpcSink<W>>) -> Fut,
    Fut: Future<Output = IoResult<RpcSink<W>>>,
  {
    let rsp = MsgPackWriter::new(self)
      .write_array_len(4)
      .await?
      .next()
      .write_int(MsgType::Response)
      .await?
      .next()
      .write_int(msgid)
      .await?
      .next()
      .write_nil()
      .await?
      .last();
    write_ok(rsp).await
  }

  pub async fn write_err_response<F, Fut>(self, msgid: MsgId, write_err: F) -> IoResult<RpcSink<W>>
  where
    F: FnOnce(MsgPackWriter<ArrayFuture<RpcSink<W>>>) -> Fut,
    Fut: Future<Output = IoResult<ArrayFuture<RpcSink<W>>>>,
  {
    let err = MsgPackWriter::new(self)
      .write_array_len(4)
      .await?
      .next()
      .write_int(MsgType::Response)
      .await?
      .next()
      .write_int(msgid)
      .await?
      .next();
    let ok = write_err(err).await?;
    ok.last().write_nil().await
  }

  #[must_use = "dropping the writer may leave the message unfinished"]
  pub async fn write_notification(
    self,
    method: impl AsRef<str>,
    num_args: u32,
  ) -> IoResult<ArrayFuture<RpcSink<W>>> {
    let args = MsgPackWriter::new(self)
      .write_array_len(3)
      .await?
      .next()
      .write_int(MsgType::Notification)
      .await?
      .next()
      .write_str(method.as_ref())
      .await?
      .last();
    args.write_array_len(num_args).await
  }
}

#[test]
fn write_request_response() {
  let sink = RpcSink::new(Vec::new());
  let f = async {
    sink
      .write_request(2.into(), "floop", 1)
      .await
      .unwrap()
      .last()
      .write_int(42)
      .await
      .unwrap()
  };

  let sink = futures::executor::LocalPool::new().run_until(f);
  let v1 = sink.into_inner();

  let mut v2 = Vec::new();
  rmp::encode::write_array_len(&mut v2, 4).unwrap();
  rmp::encode::write_uint(&mut v2, 0).unwrap();
  rmp::encode::write_uint(&mut v2, 2).unwrap();
  rmp::encode::write_str(&mut v2, "floop").unwrap();
  rmp::encode::write_array_len(&mut v2, 1).unwrap();
  rmp::encode::write_uint(&mut v2, 42).unwrap();
  assert_eq!(v1, v2);
}

#[test]
fn write_ok_response() {
  let sink = RpcSink::new(Vec::new());
  let f = sink.write_ok_response(2.into(), |rsp| rsp.write_int(42));

  let sink = futures::executor::LocalPool::new().run_until(f).unwrap();
  let v1 = sink.into_inner();

  let mut v2 = Vec::new();
  rmp::encode::write_array_len(&mut v2, 4).unwrap();
  rmp::encode::write_uint(&mut v2, 1).unwrap();
  rmp::encode::write_uint(&mut v2, 2).unwrap();
  rmp::encode::write_nil(&mut v2).unwrap();
  rmp::encode::write_uint(&mut v2, 42).unwrap();
  assert_eq!(v1, v2);
}

#[test]
fn write_err_response() {
  let sink = RpcSink::new(Vec::new());
  let f = sink.write_err_response(2.into(), |err| err.write_int(42));

  let sink = futures::executor::LocalPool::new().run_until(f).unwrap();
  let v1 = sink.into_inner();

  let mut v2 = Vec::new();
  rmp::encode::write_array_len(&mut v2, 4).unwrap();
  rmp::encode::write_uint(&mut v2, 1).unwrap();
  rmp::encode::write_uint(&mut v2, 2).unwrap();
  rmp::encode::write_uint(&mut v2, 42).unwrap();
  rmp::encode::write_nil(&mut v2).unwrap();
  assert_eq!(v1, v2);
}
