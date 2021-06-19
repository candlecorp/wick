#[cfg(feature = "read-initializer")]
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{Future, FutureExt, TryFutureExt};
use rmp::Marker;
use rmpv::{Integer, Value};

use byteorder::{BigEndian, ByteOrder};
use futures::io::ErrorKind;
use futures::io::Result as IoResult;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;

use crate::MsgPackOption;

type DynIoResultFuture<'a, T> = dyn Future<Output = IoResult<T>> + Send + 'a;

pub(crate) trait WrapReader<R, R2> {
  type Output;

  /// Convert the underlying reader into some other type that implements read.
  ///
  /// Intended for wrapping the reader to control what the client gets back
  /// after consuming this message. Can be used to make sure cleanup actions
  /// happen.
  fn wrap(self, wrap: impl FnOnce(R) -> R2) -> Self::Output;
}

#[must_use]
#[derive(Debug)]
pub enum ValueFuture<R> {
  Nil(R),
  Boolean(bool, R),
  Integer(Integer, R),
  F32(f32, R),
  F64(f64, R),
  Array(ArrayFuture<R>),
  Map(MapFuture<R>),
  Bin(BinFuture<R>),
  String(StringFuture<R>),
  Ext(ExtFuture<R>),
}

impl<R, R2> WrapReader<R, R2> for ValueFuture<R> {
  type Output = ValueFuture<R2>;

  fn wrap(self, wrap: impl FnOnce(R) -> R2) -> Self::Output {
    match self {
      ValueFuture::Nil(r) => ValueFuture::Nil(wrap(r)),
      ValueFuture::Boolean(v, r) => ValueFuture::Boolean(v, wrap(r)),
      ValueFuture::Integer(v, r) => ValueFuture::Integer(v, wrap(r)),
      ValueFuture::F32(v, r) => ValueFuture::F32(v, wrap(r)),
      ValueFuture::F64(v, r) => ValueFuture::F64(v, wrap(r)),
      ValueFuture::Array(v) => ValueFuture::Array(v.wrap(wrap)),
      ValueFuture::Map(v) => ValueFuture::Map(v.wrap(wrap)),
      ValueFuture::Bin(v) => ValueFuture::Bin(v.wrap(wrap)),
      ValueFuture::String(v) => ValueFuture::String(v.wrap(wrap)),
      ValueFuture::Ext(v) => ValueFuture::Ext(v.wrap(wrap)),
    }
  }
}

impl<R> ValueFuture<R> {
  #[must_use]
  pub fn into_bool(self) -> Option<(bool, R)> {
    if let ValueFuture::Boolean(val, r) = self {
      Some((val, r))
    } else {
      None
    }
  }

  #[must_use]
  pub fn into_bin(self) -> Option<BinFuture<R>> {
    if let ValueFuture::Bin(bin) = self {
      Some(bin)
    } else {
      None
    }
  }

  #[must_use]
  pub fn into_string(self) -> Option<StringFuture<R>> {
    if let ValueFuture::String(s) = self {
      Some(s)
    } else {
      None
    }
  }

  #[must_use]
  pub fn into_ext(self) -> Option<ExtFuture<R>> {
    if let ValueFuture::Ext(ext) = self {
      Some(ext)
    } else {
      None
    }
  }

  #[must_use]
  pub fn into_array(self) -> Option<ArrayFuture<R>> {
    if let ValueFuture::Array(array) = self {
      Some(array)
    } else {
      None
    }
  }

  #[must_use]
  pub fn into_map(self) -> Option<MapFuture<R>> {
    if let ValueFuture::Map(map) = self {
      Some(map)
    } else {
      None
    }
  }

  #[must_use]
  pub fn into_u64(self) -> Option<(u64, R)> {
    if let ValueFuture::Integer(val, r) = self {
      val.as_u64().map(|val| (val, r))
    } else {
      None
    }
  }

  #[must_use]
  pub fn into_f32(self) -> Option<(f32, R)> {
    if let ValueFuture::F32(val, r) = self {
      Some((val, r))
    } else {
      None
    }
  }

  #[must_use]
  pub fn into_f64(self) -> Option<(f64, R)> {
    if let ValueFuture::F64(val, r) = self {
      Some((val, r))
    } else {
      None
    }
  }

  pub async fn skip(self) -> IoResult<R>
  where
    R: AsyncRead + Send + Unpin,
  {
    Ok(match self {
      ValueFuture::Nil(r) => r,
      ValueFuture::Boolean(_b, r) => r,
      ValueFuture::Integer(_i, r) => r,
      ValueFuture::F32(_f, r) => r,
      ValueFuture::F64(_f, r) => r,
      ValueFuture::Array(a) => a.skip().await?,
      ValueFuture::Map(m) => m.skip().await?,
      ValueFuture::Bin(m) => m.skip().await?,
      ValueFuture::String(s) => s.skip().await?,
      ValueFuture::Ext(e) => e.skip().await?,
    })
  }

  /// Read an entire message into a heap-allocated dynamic `Value`
  pub async fn into_value(self) -> IoResult<(Value, R)>
  where
    R: AsyncRead + Send + Unpin,
  {
    // Boxing the future is necessary for array and map which recurse.
    Ok(match self {
      ValueFuture::Nil(r) => (Value::Nil, r),
      ValueFuture::Boolean(b, r) => (Value::Boolean(b), r),
      ValueFuture::Integer(i, r) => (Value::Integer(i), r),
      ValueFuture::F32(f, r) => (Value::F32(f), r),
      ValueFuture::F64(f, r) => (Value::F64(f), r),
      ValueFuture::Array(a) => a.into_value().await?,
      ValueFuture::Map(m) => m.into_value().await?,
      ValueFuture::Bin(m) => m.into_value().await?,
      ValueFuture::String(s) => s.into_value().await?,
      ValueFuture::Ext(e) => e.into_value().await?,
    })
  }
}

#[must_use]
pub struct MsgPackFuture<R> {
  reader: R,
}

impl<R: AsyncRead + Unpin> MsgPackFuture<R> {
  pub fn new(reader: R) -> Self {
    MsgPackFuture { reader }
  }

  /// Return the underlying reader without reading from it
  pub fn into_inner(self) -> R {
    self.reader
  }

  async fn read_1(&mut self) -> IoResult<u8> {
    let mut val = [0];
    self.reader.read_exact(&mut val[..]).await?;
    Ok(val[0])
  }

  async fn read_2(&mut self) -> IoResult<[u8; 2]> {
    #[cfg(feature = "read-initializer")]
    #[allow(clippy::uninit_assumed_init)]
    unsafe {
      let mut val = MaybeUninit::<[u8; 2]>::uninit().assume_init();
      self.reader.initializer().initialize(&mut val[..]);
      self.reader.read_exact(&mut val[..]).await?;
      Ok(val)
    }
    #[cfg(not(feature = "read-initializer"))]
    {
      let mut val = [0; 2];
      self.reader.read_exact(&mut val[..]).await?;
      Ok(val)
    }
  }

  async fn read_4(&mut self) -> IoResult<[u8; 4]> {
    #[cfg(feature = "read-initializer")]
    #[allow(clippy::uninit_assumed_init)]
    unsafe {
      let mut val = MaybeUninit::<[u8; 4]>::uninit().assume_init();
      self.reader.initializer().initialize(&mut val);
      self.reader.read_exact(&mut val[..]).await?;
      Ok(val)
    }
    #[cfg(not(feature = "read-initializer"))]
    {
      let mut val = [0; 4];
      self.reader.read_exact(&mut val[..]).await?;
      Ok(val)
    }
  }

  async fn read_8(&mut self) -> IoResult<[u8; 8]> {
    #[cfg(feature = "read-initializer")]
    #[allow(clippy::uninit_assumed_init)]
    unsafe {
      let mut val = MaybeUninit::<[u8; 8]>::uninit().assume_init();
      self.reader.initializer().initialize(&mut val);
      self.reader.read_exact(&mut val[..]).await?;
      Ok(val)
    }
    #[cfg(not(feature = "read-initializer"))]
    {
      let mut val = [0; 8];
      self.reader.read_exact(&mut val[..]).await?;
      Ok(val)
    }
  }

  async fn read_u8(&mut self) -> IoResult<u8> {
    self.read_1().await
  }

  async fn read_u16(&mut self) -> IoResult<u16> {
    Ok(BigEndian::read_u16(&self.read_2().await?))
  }

  async fn read_u32(&mut self) -> IoResult<u32> {
    Ok(BigEndian::read_u32(&self.read_4().await?))
  }

  async fn read_u64(&mut self) -> IoResult<u64> {
    Ok(BigEndian::read_u64(&self.read_8().await?))
  }

  async fn read_i8(&mut self) -> IoResult<i8> {
    self.read_1().await.map(|x| x as i8)
  }

  async fn read_i16(&mut self) -> IoResult<i16> {
    Ok(BigEndian::read_i16(&self.read_2().await?))
  }

  async fn read_i32(&mut self) -> IoResult<i32> {
    Ok(BigEndian::read_i32(&self.read_4().await?))
  }

  async fn read_i64(&mut self) -> IoResult<i64> {
    Ok(BigEndian::read_i64(&self.read_8().await?))
  }

  async fn read_f32(&mut self) -> IoResult<f32> {
    Ok(BigEndian::read_f32(&self.read_4().await?))
  }

  async fn read_f64(&mut self) -> IoResult<f64> {
    Ok(BigEndian::read_f64(&self.read_8().await?))
  }

  pub async fn skip(self) -> IoResult<R>
  where
    R: Send,
  {
    self.decode().await?.skip().await
  }

  pub fn skip_dyn<'a>(self) -> Pin<Box<DynIoResultFuture<'a, R>>>
  where
    R: Send + 'a,
  {
    self.skip().boxed()
  }

  pub async fn decode(mut self) -> IoResult<ValueFuture<R>> {
    let marker = Marker::from_u8(self.read_u8().await?);
    Ok(match marker {
      Marker::FixPos(val) => ValueFuture::Integer(Integer::from(val), self.reader),
      Marker::FixNeg(val) => ValueFuture::Integer(Integer::from(val), self.reader),
      Marker::Null => ValueFuture::Nil(self.reader),
      Marker::True => ValueFuture::Boolean(true, self.reader),
      Marker::False => ValueFuture::Boolean(false, self.reader),
      Marker::U8 => ValueFuture::Integer(Integer::from(self.read_u8().await?), self.reader),
      Marker::U16 => ValueFuture::Integer(Integer::from(self.read_u16().await?), self.reader),
      Marker::U32 => ValueFuture::Integer(Integer::from(self.read_u32().await?), self.reader),
      Marker::U64 => ValueFuture::Integer(Integer::from(self.read_u64().await?), self.reader),
      Marker::I8 => ValueFuture::Integer(Integer::from(self.read_i8().await?), self.reader),
      Marker::I16 => ValueFuture::Integer(Integer::from(self.read_i16().await?), self.reader),
      Marker::I32 => ValueFuture::Integer(Integer::from(self.read_i32().await?), self.reader),
      Marker::I64 => ValueFuture::Integer(Integer::from(self.read_i64().await?), self.reader),
      Marker::F32 => ValueFuture::F32(self.read_f32().await?, self.reader),
      Marker::F64 => ValueFuture::F64(self.read_f64().await?, self.reader),
      Marker::FixStr(len) => ValueFuture::String(StringFuture(BinFuture {
        reader: self.reader,
        len: len.into(),
      })),
      Marker::Str8 => {
        let len = self.read_u8().await?;
        ValueFuture::String(StringFuture(BinFuture {
          reader: self.reader,
          len: len.into(),
        }))
      }
      Marker::Str16 => {
        let len = self.read_u16().await?;
        ValueFuture::String(StringFuture(BinFuture {
          reader: self.reader,
          len: len.into(),
        }))
      }
      Marker::Str32 => {
        let len = self.read_u32().await?;
        ValueFuture::String(StringFuture(BinFuture {
          reader: self.reader,
          len: len as usize,
        }))
      }
      Marker::Bin8 => {
        let len = self.read_u8().await?;
        ValueFuture::Bin(BinFuture {
          reader: self.reader,
          len: len.into(),
        })
      }
      Marker::Bin16 => {
        let len = self.read_u16().await?.into();
        ValueFuture::Bin(BinFuture {
          reader: self.reader,
          len,
        })
      }
      Marker::Bin32 => {
        let len = self.read_u32().await? as usize;
        ValueFuture::Bin(BinFuture {
          reader: self.reader,
          len,
        })
      }
      Marker::FixArray(len) => ValueFuture::Array(ArrayFuture {
        reader: self.reader,
        len: len.into(),
      }),
      Marker::Array16 => {
        let len = self.read_u16().await?;
        ValueFuture::Array(ArrayFuture {
          reader: self.reader,
          len: len.into(),
        })
      }
      Marker::Array32 => {
        let len = self.read_u32().await?;
        ValueFuture::Array(ArrayFuture {
          reader: self.reader,
          len: len as usize,
        })
      }
      Marker::FixMap(len) => ValueFuture::Map(MapFuture {
        reader: self.reader,
        len: len.into(),
      }),
      Marker::Map16 => {
        let len = self.read_u16().await?;
        ValueFuture::Map(MapFuture {
          reader: self.reader,
          len: len.into(),
        })
      }
      Marker::Map32 => {
        let len = self.read_u32().await?;
        ValueFuture::Map(MapFuture {
          reader: self.reader,
          len: len as usize,
        })
      }
      Marker::FixExt1 => {
        let ty = self.read_i8().await?;
        ValueFuture::Ext(ExtFuture {
          bin: BinFuture {
            reader: self.reader,
            len: 1,
          },
          ty,
        })
      }
      Marker::FixExt2 => {
        let ty = self.read_i8().await?;
        ValueFuture::Ext(ExtFuture {
          bin: BinFuture {
            reader: self.reader,
            len: 2,
          },
          ty,
        })
      }
      Marker::FixExt4 => {
        let ty = self.read_i8().await?;
        ValueFuture::Ext(ExtFuture {
          bin: BinFuture {
            reader: self.reader,
            len: 4,
          },
          ty,
        })
      }
      Marker::FixExt8 => {
        let ty = self.read_i8().await?;
        ValueFuture::Ext(ExtFuture {
          bin: BinFuture {
            reader: self.reader,
            len: 8,
          },
          ty,
        })
      }
      Marker::FixExt16 => {
        let ty = self.read_i8().await?;
        ValueFuture::Ext(ExtFuture {
          bin: BinFuture {
            reader: self.reader,
            len: 16,
          },
          ty,
        })
      }
      Marker::Ext8 => {
        let len = self.read_u8().await?;
        let ty = self.read_i8().await?;
        ValueFuture::Ext(ExtFuture {
          bin: BinFuture {
            reader: self.reader,
            len: len.into(),
          },
          ty,
        })
      }
      Marker::Ext16 => {
        let len = self.read_u16().await?;
        let ty = self.read_i8().await?;
        ValueFuture::Ext(ExtFuture {
          bin: BinFuture {
            reader: self.reader,
            len: len.into(),
          },
          ty,
        })
      }
      Marker::Ext32 => {
        let len = self.read_u32().await?;
        let ty = self.read_i8().await?;
        ValueFuture::Ext(ExtFuture {
          bin: BinFuture {
            reader: self.reader,
            len: len as usize,
          },
          ty,
        })
      }
      Marker::Reserved => return Err(ErrorKind::InvalidData.into()),
    })
  }

  /// Read an entire message into a heap-allocated dynamic `Value`
  pub async fn into_value(self) -> IoResult<(Value, R)>
  where
    R: Send,
  {
    // Boxing the future is necessary for array and map which recurse.
    self.decode().await?.into_value().await
  }

  pub fn into_value_dyn<'a>(self) -> Pin<Box<DynIoResultFuture<'a, (Value, R)>>>
  where
    R: Send + 'a,
  {
    self.into_value().boxed()
  }
}

impl<R: AsyncRead + Unpin> AsyncRead for MsgPackFuture<R> {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context,
    buf: &mut tokio::io::ReadBuf<'_>,
  ) -> Poll<Result<(), std::io::Error>> {
    R::poll_read(Pin::new(&mut self.as_mut().reader), cx, buf)
  }
}

#[derive(Debug)]
pub struct ArrayFuture<R> {
  reader: R,
  len: usize,
}

impl<R: AsyncRead + Unpin> AsyncRead for ArrayFuture<R> {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context,
    buf: &mut tokio::io::ReadBuf<'_>,
  ) -> Poll<Result<(), std::io::Error>> {
    R::poll_read(Pin::new(&mut self.as_mut().reader), cx, buf)
  }
}

impl<R, R2> WrapReader<R, R2> for ArrayFuture<R> {
  type Output = ArrayFuture<R2>;

  fn wrap(self, wrap: impl FnOnce(R) -> R2) -> Self::Output {
    ArrayFuture {
      reader: wrap(self.reader),
      len: self.len,
    }
  }
}

impl<R: AsyncRead + Unpin> ArrayFuture<R> {
  pub fn len(&self) -> usize {
    self.len
  }

  pub fn is_empty(&self) -> bool {
    self.len == 0
  }

  pub fn next(mut self) -> MsgPackOption<MsgPackFuture<Self>, R> {
    if self.len > 0 {
      self.len -= 1;
      MsgPackOption::Some(MsgPackFuture::new(self))
    } else {
      MsgPackOption::End(self.reader)
    }
  }

  #[must_use]
  pub fn next_dyn(&'_ mut self) -> Option<MsgPackFuture<&'_ mut (dyn AsyncRead + Unpin + Send)>>
  where
    R: Send,
  {
    if self.len > 0 {
      self.len -= 1;
      Some(MsgPackFuture::new(&mut self.reader))
    } else {
      None
    }
  }

  /// If this is the last element, return a future of it's value wrapped around the
  /// underlying reader. Avoids having to call `next()` a final time.
  pub fn last(self) -> MsgPackOption<MsgPackFuture<R>, R> {
    if self.len == 1 {
      MsgPackOption::Some(MsgPackFuture::new(self.reader))
    } else {
      MsgPackOption::End(self.reader)
    }
  }

  /// Consume all remaining elements and return the underlying reader
  pub async fn skip(mut self) -> IoResult<R>
  where
    R: Send,
  {
    if self.is_empty() {
      return Ok(self.reader);
    }
    while let Some(m) = self.next_dyn() {
      m.skip_dyn().await?;
    }
    Ok(self.reader)
  }

  pub async fn into_value_vec(mut self) -> IoResult<(Vec<Value>, R)>
  where
    R: Send,
  {
    let mut v = Vec::with_capacity(self.len());
    while let Some(m) = self.next_dyn() {
      let (value, _) = m.into_value_dyn().await?;
      v.push(value);
    }
    Ok((v, self.reader))
  }

  pub fn into_value(self) -> impl Future<Output = IoResult<(Value, R)>>
  where
    R: Send,
  {
    self.into_value_vec().map_ok(|(v, r)| (Value::Array(v), r))
  }
}

/// Container that ensures the response message array is consumed before
/// returning the underlying reader
pub struct FinalizedArray<R>(ArrayFuture<R>);

impl<R: AsyncRead + Unpin + 'static> FinalizedArray<R> {
  pub async fn finish(self) -> IoResult<R>
  where
    R: Send,
  {
    self.0.skip().await
  }
}

impl<R: AsyncRead + Unpin> AsyncRead for FinalizedArray<R> {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context,
    buf: &mut tokio::io::ReadBuf<'_>,
  ) -> Poll<Result<(), std::io::Error>> {
    ArrayFuture::poll_read(Pin::new(&mut self.as_mut().0), cx, buf)
  }
}

#[derive(Debug)]
pub struct MapFuture<R> {
  reader: R,
  len: usize,
}

impl<R: AsyncRead + Unpin> AsyncRead for MapFuture<R> {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context,
    buf: &mut tokio::io::ReadBuf<'_>,
  ) -> Poll<Result<(), std::io::Error>> {
    R::poll_read(Pin::new(&mut self.as_mut().reader), cx, buf)
  }
}

impl<R, R2> WrapReader<R, R2> for MapFuture<R> {
  type Output = MapFuture<R2>;

  fn wrap(self, wrap: impl FnOnce(R) -> R2) -> Self::Output {
    MapFuture {
      reader: wrap(self.reader),
      len: self.len,
    }
  }
}

impl<R: AsyncRead + Unpin> MapFuture<R> {
  pub fn len(&self) -> usize {
    self.len
  }

  pub fn is_empty(&self) -> bool {
    self.len == 0
  }

  pub fn next_key(mut self) -> MsgPackOption<MsgPackFuture<MsgPackFuture<Self>>, R> {
    if self.len > 0 {
      self.len -= 1;
      MsgPackOption::Some(MsgPackFuture::new(MsgPackFuture::new(self)))
    } else {
      MsgPackOption::End(self.reader)
    }
  }

  #[must_use]
  pub fn next_key_dyn(
    &'_ mut self,
  ) -> Option<MsgPackFuture<MsgPackFuture<&'_ mut (dyn AsyncRead + Unpin + Send)>>>
  where
    R: Send,
  {
    if self.len > 0 {
      self.len -= 1;
      Some(MsgPackFuture::new(MsgPackFuture::new(&mut self.reader)))
    } else {
      None
    }
  }

  /// Consume all remaining elements and return the underlying reader
  pub async fn skip(mut self) -> IoResult<R>
  where
    R: Send,
  {
    if self.is_empty() {
      return Ok(self.reader);
    }
    while let Some(m) = self.next_key_dyn() {
      let val = m.skip_dyn().await?;
      val.skip_dyn().await?;
    }
    Ok(self.reader)
  }

  pub async fn into_value(mut self) -> IoResult<(Value, R)>
  where
    R: Send,
  {
    let mut v = Vec::with_capacity(self.len());
    while let Some(m) = self.next_key_dyn() {
      let (key, val) = m.into_value_dyn().await?;
      let (val, _next) = val.into_value_dyn().await?;
      v.push((key, val));
    }
    Ok((Value::Map(v), self.reader))
  }
}

/// Discard n bytes from a reader
async fn reader_skip<R: AsyncRead + Unpin>(reader: &mut R, mut n: usize) -> IoResult<()> {
  let mut buf = [0; 32];
  while n > 0 {
    let to_read = std::cmp::min(n, buf.len());
    reader.read_exact(&mut buf[..to_read]).await?;
    n -= to_read;
  }
  Ok(())
}

#[derive(Debug)]
pub struct BinFuture<R> {
  reader: R,
  len: usize,
}

impl<R, R2> WrapReader<R, R2> for BinFuture<R> {
  type Output = BinFuture<R2>;

  fn wrap(self, wrap: impl FnOnce(R) -> R2) -> Self::Output {
    BinFuture {
      reader: wrap(self.reader),
      len: self.len,
    }
  }
}

impl<R: AsyncRead + Unpin> BinFuture<R> {
  pub fn len(&self) -> usize {
    self.len
  }

  pub fn is_empty(&self) -> bool {
    self.len == 0
  }

  /// Panics if buf length is less than the value returned by `len()`
  pub async fn read_all(mut self, buf: &mut [u8]) -> IoResult<R> {
    self
      .reader
      .read_exact(&mut buf[..self.len])
      .await
      .map(|_| self.reader)
  }

  pub fn into_inner(self) -> R {
    assert_eq!(self.len, 0);
    self.reader
  }

  pub async fn into_vec(mut self) -> IoResult<(Vec<u8>, R)> {
    let mut vec = vec![0u8; self.len];
    self
      .reader
      .read_exact(&mut vec[..])
      .await
      .map(|_| (vec, self.reader))
  }

  pub async fn skip(mut self) -> IoResult<R> {
    reader_skip(&mut self.reader, self.len).await?;
    Ok(self.reader)
  }

  pub async fn into_value(self) -> IoResult<(Value, R)> {
    self.into_vec().await.map(|(v, r)| (Value::Binary(v), r))
  }
}

#[derive(Debug)]
pub struct StringFuture<R>(BinFuture<R>);

impl<R, R2> WrapReader<R, R2> for StringFuture<R> {
  type Output = StringFuture<R2>;

  fn wrap(self, wrap: impl FnOnce(R) -> R2) -> Self::Output {
    StringFuture(self.0.wrap(wrap))
  }
}

impl<R: AsyncRead + Unpin> StringFuture<R> {
  pub async fn into_string(self) -> IoResult<(String, R)> {
    self.0.into_vec().await.and_then(|(v, r)| {
      String::from_utf8(v)
        .map_err(|_| ErrorKind::InvalidData.into())
        .map(|s| (s, r))
    })
  }

  pub async fn read_all(self, buf: &mut [u8]) -> IoResult<R> {
    self.0.read_all(buf).await
  }

  pub async fn skip(self) -> IoResult<R> {
    self.0.skip().await
  }

  pub async fn into_value(self) -> IoResult<(Value, R)> {
    self
      .into_string()
      .await
      .map(|(s, r)| (Value::String(s.into()), r))
  }
}

impl<R> Deref for StringFuture<R> {
  type Target = BinFuture<R>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<R> DerefMut for StringFuture<R> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[derive(Debug)]
pub struct ExtFuture<R> {
  bin: BinFuture<R>,
  ty: i8,
}

impl<R, R2> WrapReader<R, R2> for ExtFuture<R> {
  type Output = ExtFuture<R2>;

  fn wrap(self, wrap: impl FnOnce(R) -> R2) -> Self::Output {
    ExtFuture {
      bin: self.bin.wrap(wrap),
      ty: self.ty,
    }
  }
}

impl<R: AsyncRead + Unpin> ExtFuture<R> {
  pub fn ext_type(&self) -> i8 {
    self.ty
  }

  pub async fn into_vec(self) -> IoResult<(i8, Vec<u8>, R)> {
    let ty = self.ty;
    self.bin.into_vec().await.map(|(v, r)| (ty, v, r))
  }

  pub async fn read_all(self, buf: &mut [u8]) -> IoResult<R> {
    self.bin.read_all(buf).await
  }

  pub async fn skip(self) -> IoResult<R> {
    self.bin.skip().await
  }

  pub async fn into_value(self) -> IoResult<(Value, R)> {
    self
      .into_vec()
      .await
      .map(|(ty, v, r)| (Value::Ext(ty, v), r))
  }
}

impl<R> Deref for ExtFuture<R> {
  type Target = BinFuture<R>;

  fn deref(&self) -> &Self::Target {
    &self.bin
  }
}

impl<R> DerefMut for ExtFuture<R> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.bin
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use rmpv::Value;
  use std::io::Cursor;

  fn value_to_vec(val: &Value) -> Cursor<Vec<u8>> {
    let mut buf = Vec::new();
    rmpv::encode::write_value(&mut buf, val).unwrap();
    Cursor::new(buf)
  }

  #[test]
  fn bool() {
    async fn bool_test(buf: Cursor<Vec<u8>>) -> IoResult<bool> {
      let msg = MsgPackFuture::new(buf);
      let (val, _r) = msg.decode().await?.into_bool().unwrap();
      Ok(val)
    }

    let val = true.into();
    let cursor = value_to_vec(&val);
    let out = futures::executor::LocalPool::new()
      .run_until(bool_test(cursor.clone()))
      .unwrap();
    assert!(out);

    let (out, _r) = futures::executor::LocalPool::new()
      .run_until(MsgPackFuture::new(cursor).into_value())
      .unwrap();
    assert_eq!(out, val);
  }

  #[test]
  #[allow(clippy::float_cmp)] // Not doing math. Constant bytes are passed through unmodified.
  fn f32() {
    async fn f32_test(buf: Cursor<Vec<u8>>) -> IoResult<f32> {
      let msg = MsgPackFuture::new(buf);
      let (val, _r) = msg.decode().await?.into_f32().unwrap();
      Ok(val)
    }

    let val = 25.5f32.into();
    let cursor = value_to_vec(&val);
    let out = futures::executor::LocalPool::new()
      .run_until(f32_test(cursor.clone()))
      .unwrap();
    assert_eq!(out, 25.5);

    let (out, _r) = futures::executor::LocalPool::new()
      .run_until(MsgPackFuture::new(cursor).into_value())
      .unwrap();
    assert_eq!(out, val);
  }

  #[test]
  #[allow(clippy::float_cmp)] // Not doing math. Constant bytes are passed through unmodified.
  fn f64() {
    async fn f64_test(buf: Cursor<Vec<u8>>) -> IoResult<f64> {
      let msg = MsgPackFuture::new(buf);
      let (val, _r) = msg.decode().await?.into_f64().unwrap();
      Ok(val)
    }

    let val = 25.5f64.into();
    let cursor = value_to_vec(&val);
    let out = futures::executor::LocalPool::new()
      .run_until(f64_test(cursor.clone()))
      .unwrap();
    assert_eq!(out, 25.5);

    let (out, _r) = futures::executor::LocalPool::new()
      .run_until(MsgPackFuture::new(cursor).into_value())
      .unwrap();
    assert_eq!(out, val);
  }

  #[test]
  fn bin() {
    async fn bin_test(buf: Cursor<Vec<u8>>) -> IoResult<Vec<u8>> {
      let msg = MsgPackFuture::new(buf);
      let (val, _r) = msg.decode().await?.into_bin().unwrap().into_vec().await?;
      Ok(val)
    }

    let val = vec![1u8, 2, 3, 4].into();
    let cursor = value_to_vec(&val);
    let out = futures::executor::LocalPool::new()
      .run_until(bin_test(cursor.clone()))
      .unwrap();
    assert_eq!(out, vec![1u8, 2, 3, 4]);

    let (out, _r) = futures::executor::LocalPool::new()
      .run_until(MsgPackFuture::new(cursor).into_value())
      .unwrap();
    assert_eq!(out, val);
  }

  #[test]
  fn string() {
    async fn string_test(buf: Cursor<Vec<u8>>) -> IoResult<String> {
      let msg = MsgPackFuture::new(buf);
      let (val, _r) = msg
        .decode()
        .await?
        .into_string()
        .unwrap()
        .into_string()
        .await?;
      Ok(val)
    }

    let val = "Hello world".into();
    let cursor = value_to_vec(&val);
    let out = futures::executor::LocalPool::new()
      .run_until(string_test(cursor.clone()))
      .unwrap();
    assert_eq!(out, "Hello world");

    let (out, _r) = futures::executor::LocalPool::new()
      .run_until(MsgPackFuture::new(cursor).into_value())
      .unwrap();
    assert_eq!(out, val);
  }

  #[test]
  fn array_int() {
    async fn array_test(buf: Cursor<Vec<u8>>) -> IoResult<Vec<u64>> {
      let msg = MsgPackFuture::new(buf);

      let mut vec = Vec::new();
      let mut array = msg.decode().await?.into_array().unwrap();

      while let MsgPackOption::Some(elem) = array.next() {
        let (elem, a) = elem.decode().await?.into_u64().unwrap();
        vec.push(elem);
        array = a;
      }

      Ok(vec)
    }
    let val = Value::from(vec![Value::from(1), 2.into(), 3.into(), 4.into()]);
    let cursor = value_to_vec(&val);
    let out = futures::executor::LocalPool::new()
      .run_until(array_test(cursor.clone()))
      .unwrap();
    assert_eq!(out, vec![1, 2, 3, 4]);

    let (out, _r) = futures::executor::LocalPool::new()
      .run_until(MsgPackFuture::new(cursor).into_value())
      .unwrap();
    assert_eq!(out, val);
  }

  #[test]
  fn map_int() {
    async fn map_test(buf: Cursor<Vec<u8>>) -> IoResult<Vec<(u64, u64)>> {
      let msg = MsgPackFuture::new(buf);

      let mut vec = Vec::new();
      let mut map = msg.decode().await?.into_map().unwrap();

      while let MsgPackOption::Some(elem) = map.next_key() {
        let (key, r) = elem.decode().await?.into_u64().unwrap();
        let (val, m) = r.decode().await?.into_u64().unwrap();
        vec.push((key, val));
        map = m;
      }

      Ok(vec)
    }
    let val = Value::from(vec![
      (Value::from(1), Value::from(2)),
      (Value::from(3), Value::from(4)),
    ]);
    let cursor = value_to_vec(&val);
    let out = futures::executor::LocalPool::new()
      .run_until(map_test(cursor.clone()))
      .unwrap();
    assert_eq!(out, vec![(1, 2), (3, 4)]);
    let (out, _r) = futures::executor::LocalPool::new()
      .run_until(MsgPackFuture::new(cursor).into_value())
      .unwrap();
    assert_eq!(out, val);
  }

  #[test]
  fn ext() {
    async fn ext_test(buf: Cursor<Vec<u8>>) -> IoResult<(i8, Vec<u8>)> {
      let msg = MsgPackFuture::new(buf);
      let (ty, val, _r) = msg.decode().await?.into_ext().unwrap().into_vec().await?;
      Ok((ty, val))
    }

    let val = Value::Ext(42, vec![1u8, 2, 3, 4]);
    let cursor = value_to_vec(&val);
    let out = futures::executor::LocalPool::new()
      .run_until(ext_test(cursor.clone()))
      .unwrap();
    assert_eq!(out, (42, vec![1, 2, 3, 4]));

    let (out, _r) = futures::executor::LocalPool::new()
      .run_until(MsgPackFuture::new(cursor).into_value())
      .unwrap();
    assert_eq!(out, val);
  }
}
