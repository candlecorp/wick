use std::convert::TryFrom;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::io::Error as IoError;
use futures::io::ErrorKind;
use futures::io::Result as IoResult;
use futures::Future;
use num_traits::FromPrimitive;
use tokio::io::AsyncRead;

use super::*;
use crate::decode::{ArrayFuture, MsgPackFuture, StringFuture, ValueFuture};

pub enum RpcMessage<R> {
  Request(RpcRequestFuture<R>),
  Response(RpcResponseFuture<R>),
  Notify(RpcNotifyFuture<R>),
}

impl<R> RpcMessage<R> {
  fn request(id: MsgId, array: ArrayFuture<R>) -> Self {
    RpcMessage::Request(RpcRequestFuture { array, id })
  }

  fn response(id: MsgId, array: ArrayFuture<R>) -> Self {
    RpcMessage::Response(RpcResponseFuture { array, id })
  }

  fn notify(array: ArrayFuture<R>) -> Self {
    RpcMessage::Notify(RpcNotifyFuture { array })
  }
}

#[derive(Debug)]
pub struct RpcRequestFuture<R> {
  array: ArrayFuture<R>,
  id: MsgId,
}

impl<R: AsyncRead + Unpin> RpcRequestFuture<R> {
  pub fn id(&self) -> MsgId {
    self.id
  }

  pub async fn method(self) -> IoResult<StringFuture<RpcParamsFuture<R>>> {
    self
      .array
      .next()
      .into_option()
      // Wrap with RpcParamsFuture before potentially returning the ValueFuture
      .map(|m| MsgPackFuture::new(RpcParamsFuture(m.into_inner())))
      .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "array missing method field"))?
      .decode()
      .await?
      .into_string()
      .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "expected method string"))
  }

  pub async fn skip(self) -> IoResult<R>
  where
    R: Send + 'static,
  {
    self
      .method()
      .await?
      .skip()
      .await?
      .params()
      .await?
      .skip()
      .await
  }
}

pub struct RpcParamsFuture<R>(ArrayFuture<R>);

impl<R: AsyncRead + Unpin> RpcParamsFuture<R> {
  pub async fn params(self) -> IoResult<ArrayFuture<R>> {
    self
      .0
      .last()
      .into_option()
      .ok_or_else(|| {
        IoError::new(
          ErrorKind::InvalidData,
          "array missing params or too many fields",
        )
      })?
      .decode()
      .await?
      .into_array()
      .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "expected params array"))
  }
}

impl<R: AsyncRead + Unpin> AsyncRead for RpcParamsFuture<R> {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context,
    buf: &mut tokio::io::ReadBuf<'_>,
  ) -> Poll<Result<(), std::io::Error>> {
    ArrayFuture::poll_read(Pin::new(&mut self.as_mut().0), cx, buf)
  }
}

#[derive(Debug)]
pub struct RpcResponseFuture<R> {
  array: ArrayFuture<R>,
  id: MsgId,
}

impl<R: AsyncRead + Unpin> RpcResponseFuture<R> {
  pub fn id(&self) -> MsgId {
    self.id
  }

  pub async fn result(
    self,
  ) -> IoResult<Result<ValueFuture<RpcResultFuture<R>>, ValueFuture<RpcResultFuture<R>>>> {
    let err = self
      .array
      .next()
      .into_option()
      // Wrap with RpcResultFuture before potentially returning the ValueFuture
      .map(|m| MsgPackFuture::new(RpcResultFuture(m.into_inner())))
      .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "array missing error field"))?
      .decode()
      .await?;
    if let ValueFuture::Nil(m) = err {
      m.0
        .next()
        .into_option()
        // Wrap with RpcResultFuture before potentially returning the ValueFuture
        .map(|m| MsgPackFuture::new(RpcResultFuture(m.into_inner())))
        .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "array missing result field"))?
        .decode()
        .await
        .map(Ok)
    } else {
      Ok(Err(err))
    }
  }

  /// Consume this message and return the underlying reader
  pub fn skip(self) -> impl Future<Output = IoResult<R>>
  where
    R: Send + 'static,
  {
    self.array.skip()
  }
}

/// Container that ensures the response message array is consumed before
/// returning the underlying reader
#[derive(Debug)]
pub struct RpcResultFuture<R>(ArrayFuture<R>);

impl<R: AsyncRead + Unpin + Send + 'static> RpcResultFuture<R> {
  pub async fn finish(self) -> IoResult<R> {
    self.0.skip().await
  }
}

impl<R: AsyncRead + Unpin> AsyncRead for RpcResultFuture<R> {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context,
    buf: &mut tokio::io::ReadBuf<'_>,
  ) -> Poll<Result<(), std::io::Error>> {
    ArrayFuture::poll_read(Pin::new(&mut self.as_mut().0), cx, buf)
  }
}

#[derive(Debug)]
pub struct RpcNotifyFuture<R> {
  array: ArrayFuture<R>,
}

impl<R: AsyncRead + Unpin + Send> RpcNotifyFuture<R> {
  pub async fn method(self) -> IoResult<StringFuture<RpcParamsFuture<R>>> {
    self
      .array
      .next()
      .into_option()
      // Wrap with RpcParamsFuture before potentially returning the ValueFuture
      .map(|m| MsgPackFuture::new(RpcParamsFuture(m.into_inner())))
      .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "array missing method field"))?
      .decode()
      .await?
      .into_string()
      .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "expected method string"))
  }

  pub async fn skip(self) -> IoResult<R>
  where
    R: 'static,
  {
    self
      .method()
      .await?
      .skip()
      .await?
      .params()
      .await?
      .skip()
      .await
  }
}

#[derive(Debug)]
pub struct RpcStream<R> {
  reader: R,
}

impl<R: AsyncRead + Unpin> RpcStream<R> {
  pub fn new(reader: R) -> Self {
    RpcStream { reader }
  }

  pub fn as_mut(&mut self) -> RpcStream<&mut R> {
    RpcStream {
      reader: &mut self.reader,
    }
  }

  /// Helper used for request and response to read the msgid field
  async fn decode_msgid<R2: AsyncRead + Unpin>(
    array: ArrayFuture<R2>,
  ) -> IoResult<(MsgId, ArrayFuture<R2>)> {
    let msgid = array
      .next()
      .into_option()
      .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "msgpack array 0-length"))?;
    let (msgid, array) = msgid
      .decode()
      .await?
      .into_u64()
      .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "msgid not int"))?;
    let msgid = u32::try_from(msgid)
      .map_err(|_| IoError::new(ErrorKind::InvalidData, "msgid out of range"))?;
    Ok((MsgId(msgid), array))
  }

  pub async fn next(self) -> IoResult<RpcMessage<RpcStream<R>>> {
    // First, wrap our RpcStream in a MsgPackFuture rather than using the
    // underlying reader. When this message is fully consumed and its reader
    // is returned, the client will be left with this RpcStream pointing at
    // the next message.
    let msg = MsgPackFuture::new(self);
    let a = msg
      .decode()
      .await?
      .into_array()
      .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "expected array"))?;
    let ty = a
      .next()
      .into_option()
      .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "msgpack array 0-length"))?;
    let (ty, array) = ty
      .decode()
      .await?
      .into_u64()
      .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "msgtype not int"))?;

    match MsgType::from_u64(ty) {
      Some(MsgType::Request) => Self::decode_msgid(array)
        .await
        .map(|(msgid, array)| RpcMessage::request(msgid, array)),
      Some(MsgType::Response) => Self::decode_msgid(array)
        .await
        .map(|(msgid, array)| RpcMessage::response(msgid, array)),
      Some(MsgType::Notification) => Ok(RpcMessage::notify(array)),
      None => Err(IoError::new(
        ErrorKind::InvalidData,
        format!("invalid msgtype {}", ty),
      )),
    }
  }

  /// Consume all messages until encountering a reply with the given id
  pub async fn wait_for_reply(
    mut self,
    msgid: MsgId,
  ) -> IoResult<
    Result<ValueFuture<RpcResultFuture<RpcStream<R>>>, ValueFuture<RpcResultFuture<RpcStream<R>>>>,
  >
  where
    R: Send + 'static,
  {
    loop {
      self = match self.next().await? {
        RpcMessage::Request(req) => req.skip().await?,
        RpcMessage::Response(rsp) => {
          if rsp.id() == msgid {
            return rsp.result().await;
          } else {
            rsp.skip().await?
          }
        }
        RpcMessage::Notify(nfy) => nfy.skip().await?,
      }
    }
  }
}

impl<R: AsyncRead + Unpin> AsyncRead for RpcStream<R> {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context,
    buf: &mut tokio::io::ReadBuf<'_>,
  ) -> Poll<Result<(), std::io::Error>> {
    R::poll_read(Pin::new(&mut self.as_mut().reader), cx, buf)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use std::io::Cursor;
  use tokio::io::AsyncRead;

  use rmpv::Value;

  #[tokio::test]
  async fn decode_request() {
    // Serialize a method call message
    let call1 = Value::Array(vec![
      0.into(),
      1.into(),
      "summon".into(),
      Value::Array(vec!["husker".into(), "knights".into()]),
    ]);
    let call2 = Value::Array(vec![
      0.into(),
      2.into(),
      "floop".into(),
      Value::Array(vec!["pig".into()]),
    ]);
    let mut buf = Vec::new();
    rmpv::encode::write_value(&mut buf, &call1).unwrap();
    rmpv::encode::write_value(&mut buf, &call2).unwrap();
    let stream = RpcStream::new(Cursor::new(buf));

    async fn read_message<R: AsyncRead + Unpin>(stream: RpcStream<R>) -> IoResult<()> {
      let stream = match stream.next().await? {
        RpcMessage::Request(req) => {
          assert_eq!(req.id(), 1.into());
          let method: StringFuture<_> = req.method().await?;
          let (method, params) = method.into_string().await?;
          assert_eq!(method, "summon");
          let params: ArrayFuture<_> = params.params().await?;
          // Read first param into heap-allocated String
          let (param1, params) = params
            .next()
            .into_option()
            .unwrap()
            .decode()
            .await?
            .into_string()
            .unwrap()
            .into_string()
            .await?;
          assert_eq!(param1, "husker");
          // Read second (last) param into fixed-length buffer
          let mut param2 = [0u8; 7];
          let stream = params
            .last()
            .into_option()
            .unwrap()
            .decode()
            .await?
            .into_string()
            .unwrap()
            .read_all(&mut param2)
            .await?;
          assert_eq!(std::str::from_utf8(&param2).unwrap(), "knights");
          stream
        }
        _ => panic!("Wrong message type"),
      };
      let _stream = match stream.next().await? {
        RpcMessage::Request(req) => {
          assert_eq!(req.id(), 2.into());
          let method: StringFuture<_> = req.method().await?;
          let (method, params) = method.into_string().await?;
          assert_eq!(method, "floop");
          let params: ArrayFuture<_> = params.params().await?;
          // Read first (last) param into fixed-length buffer
          let mut param2 = [0u8; 3];
          let stream = params
            .last()
            .into_option()
            .unwrap()
            .decode()
            .await?
            .into_string()
            .unwrap()
            .read_all(&mut param2)
            .await?;
          assert_eq!(std::str::from_utf8(&param2).unwrap(), "pig");
          stream
        }
        _ => panic!("Wrong message type"),
      };
      Ok(())
    }

    tokio::task::LocalSet::new()
      .run_until(read_message(stream))
      .await
      .expect("run");
  }
}
