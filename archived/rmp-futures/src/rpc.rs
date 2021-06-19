pub mod decode;
pub mod encode;
mod shared_reader;
mod shared_writer;

pub use shared_reader::{RequestDispatch, ResponseReceiver, RpcIncomingMessage};
pub use shared_writer::SharedRpcSink;

use crate::decode::ValueFuture;
use crate::encode::EfficientInt;
use shared_reader::RpcResultFuture;

pub type ResponseResult<R> =
  Result<ValueFuture<RpcResultFuture<R>>, ValueFuture<RpcResultFuture<R>>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MsgId(u32);

impl std::fmt::Display for MsgId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&format!("{}", self.0))
  }
}

impl From<MsgId> for EfficientInt {
  fn from(msgid: MsgId) -> EfficientInt {
    msgid.0.into()
  }
}

// Allow getting, but not modifying, the raw id
impl From<MsgId> for u32 {
  fn from(msgid: MsgId) -> u32 {
    msgid.0
  }
}

impl From<u32> for MsgId {
  fn from(id: u32) -> Self {
    MsgId(id)
  }
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Primitive)]
pub(crate) enum MsgType {
  Request = 0,
  Response = 1,
  Notification = 2,
}

impl From<MsgType> for EfficientInt {
  fn from(ty: MsgType) -> EfficientInt {
    (ty as u8).into()
  }
}
