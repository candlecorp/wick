use wasmrs_runtime::ConditionallySend;

use crate::packet_stream::BoxStream;
use crate::VPacket;

pub trait UnaryInputs<T>: ConditionallySend
where
  T: ConditionallySend,
{
  fn input(&mut self) -> &mut BoxStream<VPacket<T>>;
  fn take_input(self) -> BoxStream<VPacket<T>>;
}

pub trait BinaryInputs<L, R>: ConditionallySend
where
  L: ConditionallySend,
  R: ConditionallySend,
{
  fn left(&mut self) -> &mut BoxStream<VPacket<L>>;
  fn right(&mut self) -> &mut BoxStream<VPacket<R>>;
  fn both(self) -> (BoxStream<VPacket<L>>, BoxStream<VPacket<R>>);
}
