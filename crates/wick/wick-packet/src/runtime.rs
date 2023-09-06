use crate::{PacketStream, StreamMap};

#[must_use]
/// Turn a single [PacketStream] into multiple [PacketStream]s keyed by `ports`.
pub fn split_stream<T: IntoIterator<Item = String>>(stream: PacketStream, ports: T) -> Vec<PacketStream> {
  let mut streams = StreamMap::from_stream(stream, ports);
  let ports: Vec<_> = streams.keys().cloned().collect();
  ports.iter().map(|port| streams.take(port).unwrap()).collect()
}
