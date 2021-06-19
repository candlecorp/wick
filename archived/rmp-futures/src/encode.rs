use std::convert::TryFrom;
use std::convert::TryInto;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Future;
use futures::FutureExt;
use rmp::Marker;
use rmpv::Value;

use byteorder::{BigEndian, ByteOrder};
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;
use tokio::io::Result as IoResult;

use crate::MsgPackOption;

/// The smallest representation of a uint based on its value
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EfficientInt {
  FixPos(u8),
  U8(u8),
  U16(u16),
  U32(u32),
  U64(u64),
  FixNeg(i8),
  I8(i8),
  I16(i16),
  I32(i32),
  I64(i64),
}

impl From<u8> for EfficientInt {
  fn from(val: u8) -> Self {
    if val & 0x7f == val {
      EfficientInt::FixPos(val)
    } else {
      EfficientInt::U8(val)
    }
  }
}

impl From<i8> for EfficientInt {
  fn from(val: i8) -> Self {
    if let Ok(val) = u8::try_from(val) {
      val.into()
    } else if val as u8 & 0b1110_0000 == 0b1110_0000 {
      EfficientInt::FixNeg(val)
    } else {
      EfficientInt::I8(val)
    }
  }
}

impl From<u16> for EfficientInt {
  fn from(val: u16) -> Self {
    if let Ok(val) = u8::try_from(val) {
      val.into()
    } else {
      EfficientInt::U16(val)
    }
  }
}

impl From<i16> for EfficientInt {
  fn from(val: i16) -> Self {
    if let Ok(val) = u16::try_from(val) {
      val.into()
    } else if let Ok(val) = i8::try_from(val) {
      val.into()
    } else {
      EfficientInt::I16(val)
    }
  }
}

impl From<u32> for EfficientInt {
  fn from(val: u32) -> Self {
    if let Ok(val) = u16::try_from(val) {
      val.into()
    } else {
      EfficientInt::U32(val)
    }
  }
}

impl From<i32> for EfficientInt {
  fn from(val: i32) -> Self {
    if let Ok(val) = u32::try_from(val) {
      val.into()
    } else if let Ok(val) = i16::try_from(val) {
      val.into()
    } else {
      EfficientInt::I32(val)
    }
  }
}

impl From<u64> for EfficientInt {
  fn from(val: u64) -> Self {
    if let Ok(val) = u32::try_from(val) {
      val.into()
    } else {
      EfficientInt::U64(val)
    }
  }
}

impl From<i64> for EfficientInt {
  fn from(val: i64) -> Self {
    if let Ok(val) = u64::try_from(val) {
      val.into()
    } else if let Ok(val) = i32::try_from(val) {
      val.into()
    } else {
      EfficientInt::I64(val)
    }
  }
}

#[test]
fn efficient_u8() {
  assert_eq!(EfficientInt::from(1u8), EfficientInt::FixPos(1));
  assert_eq!(EfficientInt::from(127u8), EfficientInt::FixPos(127));
  assert_eq!(EfficientInt::from(128u8), EfficientInt::U8(128));
  assert_eq!(EfficientInt::from(255u8), EfficientInt::U8(255));
}

#[test]
fn efficient_i8() {
  assert_eq!(EfficientInt::from(1i8), EfficientInt::FixPos(1));
  assert_eq!(EfficientInt::from(-1i8), EfficientInt::FixNeg(-1));
  assert_eq!(EfficientInt::from(-32i8), EfficientInt::FixNeg(-32));
  assert_eq!(EfficientInt::from(-33i8), EfficientInt::I8(-33));
  assert_eq!(EfficientInt::from(127i8), EfficientInt::FixPos(127));
  assert_eq!(EfficientInt::from(-128i8), EfficientInt::I8(-128));
}

#[test]
fn efficient_u16() {
  assert_eq!(EfficientInt::from(1u16), EfficientInt::FixPos(1));
  assert_eq!(EfficientInt::from(127u16), EfficientInt::FixPos(127));
  assert_eq!(EfficientInt::from(128u16), EfficientInt::U8(128));
  assert_eq!(EfficientInt::from(255u16), EfficientInt::U8(255));
  assert_eq!(EfficientInt::from(256u16), EfficientInt::U16(256));
  assert_eq!(EfficientInt::from(65535u16), EfficientInt::U16(65535));
}

#[test]
fn efficient_i16() {
  assert_eq!(EfficientInt::from(1i16), EfficientInt::FixPos(1));
  assert_eq!(EfficientInt::from(-1i16), EfficientInt::FixNeg(-1));
  assert_eq!(EfficientInt::from(-32i16), EfficientInt::FixNeg(-32));
  assert_eq!(EfficientInt::from(-33i16), EfficientInt::I8(-33));
  assert_eq!(EfficientInt::from(127i16), EfficientInt::FixPos(127));
  assert_eq!(EfficientInt::from(128i16), EfficientInt::U8(128));
  assert_eq!(EfficientInt::from(-128i16), EfficientInt::I8(-128));
  assert_eq!(EfficientInt::from(-129i16), EfficientInt::I16(-129));
  assert_eq!(EfficientInt::from(255i16), EfficientInt::U8(255));
  assert_eq!(EfficientInt::from(256i16), EfficientInt::U16(256));
  assert_eq!(EfficientInt::from(-32768i16), EfficientInt::I16(-32768));
}

#[test]
fn efficient_u32() {
  assert_eq!(EfficientInt::from(1u32), EfficientInt::FixPos(1));
  assert_eq!(EfficientInt::from(127u32), EfficientInt::FixPos(127));
  assert_eq!(EfficientInt::from(128u32), EfficientInt::U8(128));
  assert_eq!(EfficientInt::from(255u32), EfficientInt::U8(255));
  assert_eq!(EfficientInt::from(256u32), EfficientInt::U16(256));
  assert_eq!(EfficientInt::from(65535u32), EfficientInt::U16(65535));
  assert_eq!(EfficientInt::from(65536u32), EfficientInt::U32(65536));
  assert_eq!(
    EfficientInt::from(4_294_967_295u32),
    EfficientInt::U32(4_294_967_295)
  );
}

#[test]
fn efficient_i32() {
  assert_eq!(EfficientInt::from(1i32), EfficientInt::FixPos(1));
  assert_eq!(EfficientInt::from(-1i32), EfficientInt::FixNeg(-1));
  assert_eq!(EfficientInt::from(-32i32), EfficientInt::FixNeg(-32));
  assert_eq!(EfficientInt::from(-33i32), EfficientInt::I8(-33));
  assert_eq!(EfficientInt::from(127i32), EfficientInt::FixPos(127));
  assert_eq!(EfficientInt::from(128i32), EfficientInt::U8(128));
  assert_eq!(EfficientInt::from(-128i32), EfficientInt::I8(-128));
  assert_eq!(EfficientInt::from(-129i32), EfficientInt::I16(-129));
  assert_eq!(EfficientInt::from(255i32), EfficientInt::U8(255));
  assert_eq!(EfficientInt::from(256i32), EfficientInt::U16(256));
  assert_eq!(EfficientInt::from(-32768i32), EfficientInt::I16(-32768));
  assert_eq!(EfficientInt::from(-32769i32), EfficientInt::I32(-32769));
  assert_eq!(EfficientInt::from(65535i32), EfficientInt::U16(65535));
  assert_eq!(EfficientInt::from(65536i32), EfficientInt::U32(65536));
  assert_eq!(
    EfficientInt::from(-2_147_483_648i32),
    EfficientInt::I32(-2_147_483_648i32)
  );
}

#[test]
fn efficient_u64() {
  assert_eq!(EfficientInt::from(1u64), EfficientInt::FixPos(1));
  assert_eq!(EfficientInt::from(127u64), EfficientInt::FixPos(127));
  assert_eq!(EfficientInt::from(128u64), EfficientInt::U8(128));
  assert_eq!(EfficientInt::from(255u64), EfficientInt::U8(255));
  assert_eq!(EfficientInt::from(256u64), EfficientInt::U16(256));
  assert_eq!(EfficientInt::from(65535u64), EfficientInt::U16(65535));
  assert_eq!(EfficientInt::from(65536u64), EfficientInt::U32(65536));
  assert_eq!(
    EfficientInt::from(4_294_967_295u64),
    EfficientInt::U32(4_294_967_295)
  );
  assert_eq!(
    EfficientInt::from(4_294_967_296u64),
    EfficientInt::U64(4_294_967_296)
  );
  assert_eq!(
    EfficientInt::from(std::u64::MAX),
    EfficientInt::U64(std::u64::MAX)
  );
}

#[test]
fn efficient_i64() {
  assert_eq!(EfficientInt::from(1i64), EfficientInt::FixPos(1));
  assert_eq!(EfficientInt::from(-1i64), EfficientInt::FixNeg(-1));
  assert_eq!(EfficientInt::from(-32i64), EfficientInt::FixNeg(-32));
  assert_eq!(EfficientInt::from(-33i64), EfficientInt::I8(-33));
  assert_eq!(EfficientInt::from(127i64), EfficientInt::FixPos(127));
  assert_eq!(EfficientInt::from(128i64), EfficientInt::U8(128));
  assert_eq!(EfficientInt::from(-128i64), EfficientInt::I8(-128));
  assert_eq!(EfficientInt::from(-129i64), EfficientInt::I16(-129));
  assert_eq!(EfficientInt::from(255i64), EfficientInt::U8(255));
  assert_eq!(EfficientInt::from(256i64), EfficientInt::U16(256));
  assert_eq!(EfficientInt::from(-32768i64), EfficientInt::I16(-32768));
  assert_eq!(EfficientInt::from(-32769i64), EfficientInt::I32(-32769));
  assert_eq!(EfficientInt::from(65535i64), EfficientInt::U16(65535));
  assert_eq!(EfficientInt::from(65536i64), EfficientInt::U32(65536));
  assert_eq!(
    EfficientInt::from(-2_147_483_648i64),
    EfficientInt::I32(-2_147_483_648i32)
  );
  assert_eq!(
    EfficientInt::from(4_294_967_295i64),
    EfficientInt::U32(4_294_967_295)
  );
  assert_eq!(
    EfficientInt::from(4_294_967_296i64),
    EfficientInt::U64(4_294_967_296)
  );
  assert_eq!(
    EfficientInt::from(std::i64::MIN),
    EfficientInt::I64(std::i64::MIN)
  );
}

#[must_use = "dropping the writer may leave the message unfinished"]
pub struct MsgPackWriter<W> {
  writer: W,
}

impl<W: AsyncWrite + Unpin> MsgPackWriter<W> {
  pub fn new(writer: W) -> Self {
    MsgPackWriter { writer }
  }

  pub fn into_inner(self) -> W {
    self.writer
  }

  async fn write_1(&mut self, val: [u8; 1]) -> IoResult<()> {
    self.writer.write_all(&val).await
  }

  async fn write_2(&mut self, val: [u8; 2]) -> IoResult<()> {
    self.writer.write_all(&val).await
  }

  async fn write_4(&mut self, val: [u8; 4]) -> IoResult<()> {
    self.writer.write_all(&val).await
  }

  async fn write_8(&mut self, val: [u8; 8]) -> IoResult<()> {
    self.writer.write_all(&val).await
  }

  async fn write_u8(&mut self, val: u8) -> IoResult<()> {
    let buf = [val];
    self.write_1(buf).await
  }

  async fn write_u16(&mut self, val: u16) -> IoResult<()> {
    let mut buf = [0u8; 2];
    BigEndian::write_u16(&mut buf, val);
    self.write_2(buf).await
  }

  async fn write_u32(&mut self, val: u32) -> IoResult<()> {
    let mut buf = [0u8; 4];
    BigEndian::write_u32(&mut buf, val);
    self.write_4(buf).await
  }

  async fn write_u64(&mut self, val: u64) -> IoResult<()> {
    let mut buf = [0u8; 8];
    BigEndian::write_u64(&mut buf, val);
    self.write_8(buf).await
  }

  async fn write_i8(&mut self, val: i8) -> IoResult<()> {
    let buf = [val as u8];
    self.write_1(buf).await
  }

  async fn write_i16(&mut self, val: i16) -> IoResult<()> {
    let mut buf = [0u8; 2];
    BigEndian::write_i16(&mut buf, val);
    self.write_2(buf).await
  }

  async fn write_i32(&mut self, val: i32) -> IoResult<()> {
    let mut buf = [0u8; 4];
    BigEndian::write_i32(&mut buf, val);
    self.write_4(buf).await
  }

  async fn write_i64(&mut self, val: i64) -> IoResult<()> {
    let mut buf = [0u8; 8];
    BigEndian::write_i64(&mut buf, val);
    self.write_8(buf).await
  }

  async fn write_marker(&mut self, marker: Marker) -> IoResult<()> {
    self.write_u8(marker.to_u8()).await
  }

  #[must_use = "dropping the writer may leave the message unfinished"]
  pub async fn write_nil(mut self) -> IoResult<W> {
    self.write_marker(Marker::Null).await.map(|()| self.writer)
  }

  #[must_use = "dropping the writer may leave the message unfinished"]
  pub async fn write_bool(mut self, val: bool) -> IoResult<W> {
    if val {
      self.write_marker(Marker::True)
    } else {
      self.write_marker(Marker::False)
    }
    .await
    .map(|()| self.writer)
  }

  async fn write_efficient_int(mut self, val: EfficientInt) -> IoResult<W> {
    match val {
      EfficientInt::FixPos(val) => self.write_marker(Marker::FixPos(val)).await,
      EfficientInt::U8(val) => {
        self.write_marker(Marker::U8).await?;
        self.write_u8(val).await
      }
      EfficientInt::U16(val) => {
        self.write_marker(Marker::U16).await?;
        self.write_u16(val).await
      }
      EfficientInt::U32(val) => {
        self.write_marker(Marker::U32).await?;
        self.write_u32(val).await
      }
      EfficientInt::U64(val) => {
        self.write_marker(Marker::U64).await?;
        self.write_u64(val).await
      }
      EfficientInt::FixNeg(val) => self.write_marker(Marker::FixNeg(val)).await,
      EfficientInt::I8(val) => {
        self.write_marker(Marker::I8).await?;
        self.write_i8(val).await
      }
      EfficientInt::I16(val) => {
        self.write_marker(Marker::I16).await?;
        self.write_i16(val).await
      }
      EfficientInt::I32(val) => {
        self.write_marker(Marker::I32).await?;
        self.write_i32(val).await
      }
      EfficientInt::I64(val) => {
        self.write_marker(Marker::I64).await?;
        self.write_i64(val).await
      }
    }
    .map(|()| self.writer)
  }

  /// Write any int (u8-u64,i8-i64) in the most efficient representation
  #[must_use = "dropping the writer may leave the message unfinished"]
  pub async fn write_int(self, val: impl Into<EfficientInt>) -> IoResult<W> {
    self.write_efficient_int(val.into()).await
  }

  #[must_use = "dropping the writer may leave the message unfinished"]
  pub async fn write_f32(mut self, val: f32) -> IoResult<W> {
    self.write_marker(Marker::F32).await?;
    let mut buf = [0u8; 4];
    BigEndian::write_f32(&mut buf, val);
    self.write_4(buf).await.map(|()| self.writer)
  }

  #[must_use = "dropping the writer may leave the message unfinished"]
  pub async fn write_f64(mut self, val: f64) -> IoResult<W> {
    self.write_marker(Marker::F64).await?;
    let mut buf = [0u8; 8];
    BigEndian::write_f64(&mut buf, val);
    self.write_8(buf).await.map(|()| self.writer)
  }

  #[must_use = "dropping the writer may leave the message unfinished"]
  pub async fn write_array_len(mut self, len: u32) -> IoResult<ArrayFuture<W>> {
    const U16MAX: u32 = std::u16::MAX as u32;

    match len {
      0..=15 => self.write_marker(Marker::FixArray(len as u8)).await,
      16..=U16MAX => {
        self.write_marker(Marker::Array16).await?;
        self.write_u16(len as u16).await
      }
      _ => {
        self.write_marker(Marker::Array32).await?;
        self.write_u32(len).await
      }
    }
    .map(|()| ArrayFuture {
      len: len as usize,
      writer: self.writer,
    })
  }

  #[must_use = "dropping the writer may leave the message unfinished"]
  pub async fn write_map_len(mut self, len: u32) -> IoResult<MapFuture<W>> {
    const U16MAX: u32 = std::u16::MAX as u32;

    match len {
      0..=15 => self.write_marker(Marker::FixMap(len as u8)).await,
      16..=U16MAX => {
        self.write_marker(Marker::Map16).await?;
        self.write_u16(len as u16).await
      }
      _ => {
        self.write_marker(Marker::Map32).await?;
        self.write_u32(len).await
      }
    }
    .map(|()| MapFuture {
      len: len as usize,
      writer: self.writer,
    })
  }

  /// Encodes and attempts to write the most efficient binary array length
  /// representation TODO: return binwriter
  #[must_use = "dropping the writer may leave the message unfinished"]
  pub async fn write_bin_len(mut self, len: u32) -> IoResult<W> {
    if let Ok(len) = u8::try_from(len) {
      self.write_marker(Marker::Bin8).await?;
      self.write_u8(len).await
    } else if let Ok(len) = u16::try_from(len) {
      self.write_marker(Marker::Bin16).await?;
      self.write_u16(len).await
    } else {
      self.write_marker(Marker::Bin32).await?;
      self.write_u32(len).await
    }
    .map(|()| self.writer)
  }

  /// Encodes and attempts to write the most efficient binary representation
  #[must_use = "dropping the writer may leave the message unfinished"]
  pub async fn write_bin(self, data: &[u8]) -> IoResult<W> {
    let mut w = self.write_bin_len(data.len().try_into().unwrap()).await?;
    w.write_all(data).await?;
    Ok(w)
  }

  /// Encodes and attempts to write the most efficient binary array length
  /// representation TODO: return str writer
  #[must_use = "dropping the writer may leave the message unfinished"]
  pub async fn write_str_len(mut self, len: u32) -> IoResult<W> {
    if let Ok(len) = u8::try_from(len) {
      if len < 32 {
        self.write_marker(Marker::FixStr(len)).await
      } else {
        self.write_marker(Marker::Str8).await?;
        self.write_u8(len).await
      }
    } else if let Ok(len) = u16::try_from(len) {
      self.write_marker(Marker::Str16).await?;
      self.write_u16(len).await
    } else {
      self.write_marker(Marker::Str32).await?;
      self.write_u32(len).await
    }
    .map(|()| self.writer)
  }

  /// Encodes and attempts to write the most efficient binary representation
  #[must_use = "dropping the writer may leave the message unfinished"]
  pub async fn write_str_bytes(self, string: &[u8]) -> IoResult<W> {
    let mut w = self.write_str_len(string.len().try_into().unwrap()).await?;
    w.write_all(string).await?;
    Ok(w)
  }

  /// Encodes and attempts to write the most efficient binary representation
  #[must_use = "dropping the writer may leave the message unfinished"]
  pub async fn write_str(self, string: &str) -> IoResult<W> {
    self.write_str_bytes(string.as_bytes()).await
  }

  /// Encodes and attempts to write the most efficient ext metadata
  /// representation
  ///
  /// # Panics
  ///
  /// Panics if `ty` is negative, because it is reserved for future MessagePack
  /// extension including 2-byte type information.
  #[must_use = "dropping the writer may leave the message unfinished"]
  pub async fn write_ext_meta(mut self, len: u32, ty: i8) -> IoResult<W> {
    assert!(ty >= 0);

    if let Ok(len) = u8::try_from(len) {
      match len {
        1 => {
          self.write_marker(Marker::FixExt1).await?;
        }
        2 => {
          self.write_marker(Marker::FixExt2).await?;
        }
        4 => {
          self.write_marker(Marker::FixExt4).await?;
        }
        8 => {
          self.write_marker(Marker::FixExt8).await?;
        }
        16 => {
          self.write_marker(Marker::FixExt16).await?;
        }
        len => {
          self.write_marker(Marker::Ext8).await?;
          self.write_u8(len).await?;
        }
      }
    } else if let Ok(len) = u16::try_from(len) {
      self.write_marker(Marker::Ext16).await?;
      self.write_u16(len).await?;
    } else {
      self.write_marker(Marker::Ext32).await?;
      self.write_u32(len).await?;
    }
    self.write_u8(ty as u8).await.map(|()| self.writer)
  }

  #[must_use = "dropping the writer may leave the message unfinished"]
  pub async fn write_ext(self, data: &[u8], ty: i8) -> IoResult<W> {
    let mut w = self
      .write_ext_meta(data.len().try_into().unwrap(), ty)
      .await?;
    w.write_all(data).await?;
    Ok(w)
  }

  /// Encodes and attempts to write a dynamic `rmpv::Value`
  ///
  /// # Panics
  ///
  /// Panics if array or map length exceeds 2^32-1
  #[must_use = "dropping the writer may leave the message unfinished"]
  pub async fn write_value(self, value: &Value) -> IoResult<W>
  where
    W: Send,
  {
    match value {
      Value::Nil => self.write_nil().await,
      Value::Boolean(val) => self.write_bool(*val).await,
      Value::Integer(val) => {
        if let Some(val) = val.as_i64() {
          self.write_int(val).await
        } else if let Some(val) = val.as_u64() {
          self.write_int(val).await
        } else {
          unreachable!()
        }
      }
      Value::F32(val) => self.write_f32(*val).await,
      Value::F64(val) => self.write_f64(*val).await,
      Value::String(val) => self.write_str_bytes(val.as_bytes()).await,
      Value::Binary(val) => self.write_bin(val).await,
      Value::Array(a) => {
        let w = self.write_array_len(a.len().try_into().unwrap()).await?;
        w.write_value(a).await
      }
      Value::Map(m) => {
        let w = self.write_map_len(m.len().try_into().unwrap()).await?;
        w.write_value(m).await
      }
      Value::Ext(ty, bytes) => self.write_ext(bytes, *ty).await,
    }
  }

  #[must_use = "dropping the writer may leave the message unfinished"]
  pub fn write_value_dyn<'a>(
    self,
    value: &'a Value,
  ) -> Pin<Box<dyn Future<Output = IoResult<W>> + Send + 'a>>
  where
    W: Send + 'a,
  {
    self.write_value(value).boxed()
  }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for MsgPackWriter<W> {
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

#[derive(Debug)]
pub struct ArrayFuture<W> {
  writer: W,
  len: usize,
}

impl<W: AsyncWrite + Unpin> AsyncWrite for ArrayFuture<W> {
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

impl<W: AsyncWrite + Unpin> ArrayFuture<W> {
  pub fn len(&self) -> usize {
    self.len
  }

  pub fn is_empty(&self) -> bool {
    self.len == 0
  }

  /// Return the underlying writer from this `ArrayFuture` that has already
  /// written all elements (len == 0).
  ///
  /// #Panics
  ///
  /// Panics if self.is_empty() is false. `next()` must have been called as
  /// many times as the array length originally written before calling this to
  /// destroy the array wrapper.
  pub fn end(self) -> W {
    assert!(self.is_empty());
    self.writer
  }

  /// Return a `MsgPackWriter` for the next array element
  ///
  /// Use for encoding a fixed amount of nesting of arrays/maps that can be
  /// modeled with straight-line code. For recursion or looping where the depth
  /// is not known at compile time, see `next_dyn()`.
  ///
  /// #Panics
  ///
  /// Panics if self.is_empty() is true. `next()` must be called no more times
  /// than the array length originally written.
  pub fn next(mut self) -> MsgPackWriter<Self> {
    assert!(!self.is_empty());
    self.len -= 1;
    MsgPackWriter::new(self)
  }

  /// Return a `MsgPackWriter` for the next array element
  ///
  /// Borrows from the ArrayFuture and the returned `MsgPackWriter` is based on
  /// a trait object. This is useful for recursion. It is not possible to
  /// manipulate this ArrayFuture while the `MsgPackWriter` exists, but the
  /// user must be careful to write to it before dropping it, or the output
  /// will be malformed.
  ///
  /// #Panics
  ///
  /// Panics if self.is_empty() is true. `next()` must be called no more times
  /// than the array length originally written.
  #[must_use = "dropping the writer may leave the message unfinished"]
  pub fn next_dyn(&mut self) -> MsgPackWriter<&mut (dyn AsyncWrite + Send + Unpin)>
  where
    W: Send,
  {
    assert!(!self.is_empty());
    self.len -= 1;
    MsgPackWriter::new(&mut self.writer)
  }

  /// If there is one element left, return a `MsgPackWriter` for the last array
  /// element.
  ///
  /// This will yield the underlying writer when the element is written. This
  /// avoids have to also call `end()` after writing this element.
  ///
  /// #Panics
  ///
  /// Panics if self.len() != 1. `next()` must have been called as many times
  /// as the array length originally written - 1 before calling this to destroy
  /// the array wrapper.
  pub fn last(self) -> MsgPackWriter<W> {
    assert_eq!(self.len(), 1);
    MsgPackWriter::new(self.writer)
  }

  pub async fn write_value(mut self, a: &[Value]) -> IoResult<W>
  where
    W: Send,
  {
    for elem in a {
      self.next_dyn().write_value_dyn(elem).await?;
    }
    Ok(self.end())
  }
}

#[derive(Debug)]
pub struct MapFuture<W> {
  writer: W,
  len: usize,
}

impl<W: AsyncWrite + Unpin> AsyncWrite for MapFuture<W> {
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

impl<W: AsyncWrite + Unpin> MapFuture<W> {
  pub fn len(&self) -> usize {
    self.len
  }

  pub fn is_empty(&self) -> bool {
    self.len == 0
  }

  /// Return the underlying writer from this `MapFuture` that has already
  /// written all elements (len == 0).
  ///
  /// #Panics
  ///
  /// Panics if self.is_empty() is false. `next()` must have been called as
  /// many times as the map length originally written before calling this to
  /// destroy the array wrapper.
  pub fn end(self) -> W {
    assert!(self.is_empty());
    self.writer
  }

  pub fn next_key(mut self) -> MsgPackOption<MsgPackWriter<MsgPackWriter<Self>>, W> {
    if self.len > 0 {
      self.len -= 1;
      MsgPackOption::Some(MsgPackWriter::new(MsgPackWriter::new(self)))
    } else {
      MsgPackOption::End(self.writer)
    }
  }

  #[must_use = "dropping the writer may leave the message unfinished"]
  pub fn next_key_dyn(
    &mut self,
  ) -> Option<MsgPackWriter<MsgPackWriter<&mut (dyn AsyncWrite + Send + Unpin)>>>
  where
    W: Send,
  {
    if self.len > 0 {
      self.len -= 1;
      Some(MsgPackWriter::new(MsgPackWriter::new(self)))
    } else {
      None
    }
  }

  /// If this is the last element, return a future of it's value wrapped around the
  /// underlying writer. Avoids having to call `next()` a final time.
  pub fn last_key(self) -> MsgPackOption<MsgPackWriter<MsgPackWriter<W>>, W> {
    if self.len == 1 {
      MsgPackOption::Some(MsgPackWriter::new(MsgPackWriter::new(self.writer)))
    } else {
      MsgPackOption::End(self.writer)
    }
  }

  pub async fn write_value(mut self, a: &[(Value, Value)]) -> IoResult<W>
  where
    W: Send,
  {
    for (k, v) in a {
      self
        .next_key_dyn()
        .unwrap()
        .write_value_dyn(k)
        .await?
        .write_value_dyn(v)
        .await?;
    }
    Ok(self.end())
  }
}

#[derive(Debug)]
pub struct MapValueFuture<W> {
  writer: MapFuture<W>,
}

impl<W: AsyncWrite + Unpin> AsyncWrite for MapValueFuture<W> {
  fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<IoResult<usize>> {
    MapFuture::poll_write(Pin::new(&mut self.as_mut().writer), cx, buf)
  }

  fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<IoResult<()>> {
    MapFuture::poll_flush(Pin::new(&mut self.as_mut().writer), cx)
  }

  fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<IoResult<()>> {
    MapFuture::poll_shutdown(Pin::new(&mut self.as_mut().writer), cx)
  }
}

impl<W: AsyncWrite + Unpin> MapValueFuture<W> {
  pub fn next_value(self) -> MsgPackWriter<MapFuture<W>> {
    MsgPackWriter::new(self.writer)
  }
}

#[cfg(test)]
mod tests {
  use futures::TryFutureExt;

  use super::*;

  fn run_future<R>(f: impl Future<Output = R>) -> R {
    futures::executor::LocalPool::new().run_until(f)
  }

  /// Create a 2 writable cursors and wrap one in a `MsgPackWriter` call a
  /// function to write with rmp::encode and MsgPackWriter, and return an
  /// optional rmpv::Value that will get encoded with MsgPackWriter::write_value.
  /// All three will be checked for equality.
  fn test_jig<F>(f: F)
  where
    F: FnOnce(&mut Vec<u8>, MsgPackWriter<Vec<u8>>) -> (Option<Value>, Vec<u8>),
  {
    let mut v1 = Vec::new();
    let msg2 = MsgPackWriter::new(Vec::new());
    let (val, v2) = f(&mut v1, msg2);

    assert_eq!(v1, v2);

    if let Some(val) = val {
      let msg2 = MsgPackWriter::new(Vec::new());
      // Encode the `Value`
      let v3 = run_future(msg2.write_value(&val)).unwrap();
      assert_eq!(v1, v3);
    }
  }

  #[test]
  fn nil() {
    test_jig(|c1, msg| {
      rmp::encode::write_nil(c1).unwrap();
      (Some(Value::Nil), run_future(msg.write_nil()).unwrap())
    });
  }

  #[test]
  fn bool() {
    test_jig(|c1, msg| {
      rmp::encode::write_bool(c1, true).unwrap();
      (
        Some(Value::Boolean(true)),
        run_future(msg.write_bool(true)).unwrap(),
      )
    });
    test_jig(|c1, msg| {
      rmp::encode::write_bool(c1, false).unwrap();
      (
        Some(Value::Boolean(false)),
        run_future(msg.write_bool(false)).unwrap(),
      )
    });
  }

  #[test]
  fn float() {
    test_jig(|c1, msg| {
      rmp::encode::write_f32(c1, 1.1).unwrap();
      (
        Some(Value::F32(1.1)),
        run_future(msg.write_f32(1.1)).unwrap(),
      )
    });
    test_jig(|c1, msg| {
      rmp::encode::write_f64(c1, 1.1).unwrap();
      (
        Some(Value::F64(1.1)),
        run_future(msg.write_f64(1.1)).unwrap(),
      )
    });
  }

  #[test]
  fn array_len() {
    for i in &[0, 1, 15, 16, 65535, 65536, std::u32::MAX] {
      test_jig(|c1, msg| {
        rmp::encode::write_array_len(c1, *i).unwrap();
        (None, run_future(msg.write_array_len(*i)).unwrap().writer)
      });
    }
  }

  #[test]
  fn array() {
    test_jig(|c1, msg| {
      rmp::encode::write_array_len(c1, 1).unwrap();
      rmp::encode::write_uint(c1, 1).unwrap();
      let f = msg.write_array_len(1).and_then(|a| a.last().write_int(1));
      (Some(Value::Array(vec![1.into()])), run_future(f).unwrap())
    })
  }

  #[test]
  fn map_len() {
    for i in &[0, 1, 15, 16, 65535, 65536, std::u32::MAX] {
      test_jig(|c1, msg| {
        rmp::encode::write_map_len(c1, *i).unwrap();
        (None, run_future(msg.write_map_len(*i)).unwrap().writer)
      });
    }
  }

  #[test]
  fn map() {
    test_jig(|c1, msg| {
      rmp::encode::write_map_len(c1, 1).unwrap();
      rmp::encode::write_uint(c1, 1).unwrap();
      rmp::encode::write_uint(c1, 2).unwrap();
      let f = msg
        .write_map_len(1)
        .and_then(|m| m.next_key().unwrap().write_int(1))
        .and_then(|m| m.write_int(2));
      (
        Some(Value::Map(vec![(1.into(), 2.into())])),
        run_future(f).unwrap().next_key().unwrap_end(),
      )
    })
  }

  #[test]
  fn bin() {
    for i in &[0, 1, 255, 256, 65535, 65536, std::u32::MAX] {
      test_jig(|c1, msg| {
        rmp::encode::write_bin_len(c1, *i).unwrap();
        (None, run_future(msg.write_bin_len(*i)).unwrap())
      });
    }
    test_jig(|c1, msg| {
      let buf = [1, 2, 3, 4];
      rmp::encode::write_bin(c1, &buf).unwrap();
      (
        Some(Value::Binary(buf[..].into())),
        run_future(msg.write_bin(&buf)).unwrap(),
      )
    });
  }

  #[test]
  fn ext() {
    for i in &[0, 1, 2, 4, 8, 16, 17, 255, 256, 65535, 65536, std::u32::MAX] {
      test_jig(|c1, msg| {
        rmp::encode::write_ext_meta(c1, *i, 42).unwrap();
        (None, run_future(msg.write_ext_meta(*i, 42)).unwrap())
      });
    }
  }

  #[test]
  fn string() {
    for i in &[0, 1, 31, 32, 255, 256, 65535, 65536, std::u32::MAX] {
      test_jig(|c1, msg| {
        rmp::encode::write_str_len(c1, *i).unwrap();
        (None, run_future(msg.write_str_len(*i)).unwrap())
      });
    }
    test_jig(|c1, msg| {
      rmp::encode::write_str(c1, "hello").unwrap();
      (
        Some("hello".into()),
        run_future(msg.write_str("hello")).unwrap(),
      )
    });
  }

  #[test]
  fn efficient_uint() {
    fn test_against_rmpv<V: Into<u64> + Into<EfficientInt> + Into<Value> + Copy>(val: V) {
      test_jig(|c1, msg| {
        rmp::encode::write_uint(c1, val.into()).unwrap();
        (Some(val.into()), run_future(msg.write_int(val)).unwrap())
      })
    }

    test_against_rmpv(1u8);
    test_against_rmpv(127u8);
    test_against_rmpv(128u8);
    test_against_rmpv(255u8);

    test_against_rmpv(1u16);
    test_against_rmpv(127u16);
    test_against_rmpv(128u16);
    test_against_rmpv(255u16);
    test_against_rmpv(256u16);
    test_against_rmpv(65535u16);

    test_against_rmpv(1u32);
    test_against_rmpv(127u32);
    test_against_rmpv(128u32);
    test_against_rmpv(255u32);
    test_against_rmpv(256u32);
    test_against_rmpv(65535u32);
    test_against_rmpv(65536u32);
    test_against_rmpv(4_294_967_295u32);

    test_against_rmpv(1u64);
    test_against_rmpv(127u64);
    test_against_rmpv(128u64);
    test_against_rmpv(255u64);
    test_against_rmpv(256u64);
    test_against_rmpv(65535u64);
    test_against_rmpv(65536u64);
    test_against_rmpv(4_294_967_295u64);
    test_against_rmpv(4_294_967_296u64);
    test_against_rmpv(std::u64::MAX);
  }

  #[test]
  fn efficient_int() {
    fn test_against_rmpv<V: Into<i64> + Into<EfficientInt> + Into<Value> + Copy>(val: V) {
      test_jig(|c1, msg| {
        rmp::encode::write_sint(c1, val.into()).unwrap();
        (Some(val.into()), run_future(msg.write_int(val)).unwrap())
      })
    }

    test_against_rmpv(1i8);
    test_against_rmpv(-1i8);
    test_against_rmpv(-32i8);
    test_against_rmpv(-33i8);
    test_against_rmpv(127i8);
    test_against_rmpv(-128i8);

    test_against_rmpv(1i16);
    test_against_rmpv(-1i16);
    test_against_rmpv(-32i16);
    test_against_rmpv(-33i16);
    test_against_rmpv(127i16);
    test_against_rmpv(128i16);
    test_against_rmpv(-128i16);
    test_against_rmpv(-129i16);
    test_against_rmpv(255i16);
    test_against_rmpv(256i16);
    test_against_rmpv(-32768i16);

    test_against_rmpv(1i32);
    test_against_rmpv(-1i32);
    test_against_rmpv(-32i32);
    test_against_rmpv(-33i32);
    test_against_rmpv(127i32);
    test_against_rmpv(128i32);
    test_against_rmpv(-128i32);
    test_against_rmpv(-129i32);
    test_against_rmpv(255i32);
    test_against_rmpv(256i32);
    test_against_rmpv(-32768i32);
    test_against_rmpv(-32769i32);
    test_against_rmpv(65535i32);
    test_against_rmpv(65536i32);
    test_against_rmpv(-2_147_483_648i32);

    test_against_rmpv(1i64);
    test_against_rmpv(-1i64);
    test_against_rmpv(-32i64);
    test_against_rmpv(-33i64);
    test_against_rmpv(127i64);
    test_against_rmpv(128i64);
    test_against_rmpv(-128i64);
    test_against_rmpv(-129i64);
    test_against_rmpv(255i64);
    test_against_rmpv(256i64);
    test_against_rmpv(-32768i64);
    test_against_rmpv(-32769i64);
    test_against_rmpv(65535i64);
    test_against_rmpv(65536i64);
    test_against_rmpv(-2_147_483_648i64);
    test_against_rmpv(-2_147_483_649i64);
    test_against_rmpv(4_294_967_295i64);
    test_against_rmpv(4_294_967_296i64);
    test_against_rmpv(std::i64::MIN);
  }
}
