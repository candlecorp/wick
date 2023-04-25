use crate::{PacketStream, StreamMap};

#[must_use]
/// Turn a single [PacketStream] into multiple [PacketStream]s keyed by [ports].
pub fn split_stream(stream: PacketStream, ports: &[String]) -> Vec<PacketStream> {
  let mut streams = StreamMap::from_stream(stream, ports);
  ports.iter().map(|port| streams.take(port).unwrap()).collect()
}
