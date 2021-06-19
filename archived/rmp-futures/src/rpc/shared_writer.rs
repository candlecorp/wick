use std::pin::Pin;
use std::task::{Context, Poll};

use super::encode::RpcSink;
use futures::io::Result as IoResult;
use futures::lock::{Mutex, MutexGuard};
use futures::{Future, FutureExt};
use tokio::io::AsyncWrite;

#[derive(Debug)]
pub struct SharedRpcSink<W> {
  writer: Mutex<W>,
}

impl<W: AsyncWrite + Unpin> SharedRpcSink<W> {
  pub fn new(writer: W) -> Self {
    SharedRpcSink {
      writer: Mutex::new(writer),
    }
  }

  pub fn lock(&self) -> impl Future<Output = RpcSink<RpcMutexGuard<W>>> {
    // Lock the writer and wrap it in an RpcSink
    self.writer.lock().map(RpcMutexGuard).map(RpcSink::new)
  }

  /// Consumes this `SharedRpcSink`, returning the underlying writer.
  pub fn into_inner(self) -> W {
    self.writer.into_inner()
  }
}

/// Newtype for implementing AsyncWrite for `MutexGuard<W>`
pub struct RpcMutexGuard<'a, W>(MutexGuard<'a, W>);

impl<'a, W: AsyncWrite + Unpin> AsyncWrite for RpcMutexGuard<'a, W> {
  fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<IoResult<usize>> {
    W::poll_write(Pin::new(&mut self.as_mut().0), cx, buf)
  }

  fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<IoResult<()>> {
    W::poll_flush(Pin::new(&mut self.as_mut().0), cx)
  }

  fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<IoResult<()>> {
    W::poll_shutdown(Pin::new(&mut self.as_mut().0), cx)
  }
}

#[test]
fn shared_sink() {
  use std::sync::Arc;

  let writer = Vec::new();
  let shared = Arc::new(SharedRpcSink::new(writer));
  let shared2 = shared.clone();

  // Make sure we can share the writer with another thread
  let thread = std::thread::spawn(move || {
    futures::executor::LocalPool::new()
      .run_until(
        shared
          .lock()
          .then(|rpc| rpc.write_ok_response(2.into(), |rsp| rsp.write_int(42))),
      )
      .unwrap();
  });
  futures::executor::LocalPool::new()
    .run_until(
      shared2
        .lock()
        .then(|rpc| rpc.write_ok_response(2.into(), |rsp| rsp.write_int(42))),
    )
    .unwrap();
  thread.join().unwrap();
}
