use serde::Serialize;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::{StreamExt, StreamMap};
use wasmflow_packet::v1::Packet as V1;
use wasmflow_packet::{Packet, PacketWrapper};
use wasmflow_streams::PacketStream;

type Error = Box<dyn std::error::Error + Send + Sync>;

type Result = std::result::Result<(), Error>;

/// The native PortSender trait. This trait encapsulates sending messages out of native ports.
pub trait Writable {
  /// The type of data that the port outputs.
  type PayloadType: Serialize;

  /// Get the port buffer that the sender can push to.
  fn get_port(&self) -> std::result::Result<&PortChannel, Error>;

  /// Get the port's name.
  fn get_port_name(&self) -> &str;

  /// Return the ID of the transaction.
  fn get_id(&self) -> u32;

  /// Send a message.
  fn send(&self, data: Self::PayloadType) -> Result {
    self.push(Packet::V1(V1::success(&data)))
  }

  /// Send a message then close the port.
  fn done(&self, data: Self::PayloadType) -> Result {
    self.send(data)?;
    self.send_message(Packet::V1(V1::done()))
  }

  /// Send a complete [Packet].
  fn push(&self, output: Packet) -> Result {
    self.get_port()?.send(PacketWrapper {
      payload: output,
      port: self.get_port_name().to_owned(),
    })?;
    Ok(())
  }

  /// Send a complete [Packet].
  fn send_message(&self, packet: Packet) -> Result {
    self.get_port()?.send(PacketWrapper {
      payload: packet,
      port: self.get_port_name().to_owned(),
    })?;
    Ok(())
  }

  /// Send a payload then close the port.
  fn done_message(&self, packet: Packet) -> Result {
    self.send_message(packet)?;
    self.send_message(V1::done().into())
  }

  /// Send an exception.
  fn send_exception(&self, payload: String) -> Result {
    self.get_port()?.send(PacketWrapper {
      payload: V1::exception(payload).into(),
      port: self.get_port_name().to_owned(),
    })?;
    Ok(())
  }

  /// Send an exception then close the port.
  fn done_exception(&self, payload: String) -> Result {
    self.send_exception(payload)?;
    self.send_message(V1::done().into())
  }

  /// Signal that a job is finished with the port.
  fn close(&self) -> Result {
    self.send_message(V1::done().into())
  }
}

/// A [PortChannel] wraps an unbounded channel with a port name.
#[must_use]
#[derive(Debug, Clone)]
pub struct PortChannel {
  /// Port name.
  pub name: String,
  incoming: Option<UnboundedSender<PacketWrapper>>,
}

impl PortChannel {
  /// Constructor for a [PortChannel].
  pub fn new<T: AsRef<str>>(name: T) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      incoming: None,
    }
  }

  /// Initialize the [PortChannel] and return a receiver.
  pub fn open(&mut self) -> UnboundedReceiverStream<PacketWrapper> {
    let (tx, rx) = unbounded_channel();
    self.incoming = Some(tx);
    UnboundedReceiverStream::new(rx)
  }

  /// Drop the incoming channel, closing the upstream.
  pub fn close(&mut self) {
    self.incoming.take();
  }

  /// Returns true if the port still has an active upstream.
  #[must_use]
  pub fn is_closed(&self) -> bool {
    self.incoming.is_none()
  }

  /// Send a messages to the channel.
  pub fn send(&self, msg: PacketWrapper) -> Result {
    let incoming = self
      .incoming
      .as_ref()
      .ok_or_else::<Error, _>(|| "Send channel closed".into())?;
    incoming.send(msg)?;
    Ok(())
  }

  /// Merge a list of [PortChannel]s into a TransportStream.
  pub fn merge_all(buffer: &mut [&mut PortChannel]) -> PacketStream {
    let mut channels = StreamMap::new();
    for channel in buffer {
      channels.insert(channel.name.clone(), channel.open());
    }
    let stream = channels.map(|(_, packet)| packet);

    PacketStream::new(Box::new(stream))
  }
}

#[cfg(test)]
mod tests {

  use wasmflow_transport::{TransportStream, TransportWrapper};
  use wasmflow_packet::v1::Packet;

  use super::*;
  struct StringSender {
    port: PortChannel,
  }
  impl Writable for StringSender {
    type PayloadType = String;
    fn get_port(&self) -> std::result::Result<&PortChannel, Error> {
      Ok(&self.port)
    }

    fn get_port_name(&self) -> &str {
      &self.port.name
    }

    fn get_id(&self) -> u32 {
      0
    }
  }

  struct I64Sender {
    port: PortChannel,
  }
  impl Writable for I64Sender {
    type PayloadType = i64;
    fn get_port(&self) -> std::result::Result<&PortChannel, Error> {
      Ok(&self.port)
    }

    fn get_port_name(&self) -> &str {
      &self.port.name
    }

    fn get_id(&self) -> u32 {
      0
    }
  }

  #[test_log::test(tokio::test)]
  async fn test_merge() -> Result {
    // This sets up the ports, sends data on them, then
    // drops the ports, thus closing them.
    let aggregated = {
      let mut port1 = StringSender {
        port: PortChannel::new("test1"),
      };
      let mut port2 = I64Sender {
        port: PortChannel::new("test2"),
      };

      let aggregated = PortChannel::merge_all(&mut [&mut port1.port, &mut port2.port]);

      port1.send("First".to_owned())?;
      port2.send(1i64)?;
      port1.done("Second".to_owned())?;
      port2.done(2i64)?;

      aggregated
    };
    let mut aggregated = TransportStream::new(aggregated.map(|pw| pw.into()));

    let mut messages = aggregated.drain_port("test1").await?;
    assert_eq!(messages.len(), 2);
    assert_eq!(aggregated.buffered_size(), (1, 2));
    let payload: String = messages.remove(0).deserialize().unwrap();
    println!("Payload a1: {}", payload);
    assert_eq!(payload, "First");
    let payload: String = messages.remove(0).deserialize().unwrap();
    println!("Payload a2: {}", payload);
    assert_eq!(payload, "Second");

    let mut messages = aggregated.drain_port("test2").await?;
    assert_eq!(messages.len(), 2);
    assert_eq!(aggregated.buffered_size(), (0, 0));
    let payload: i64 = messages.remove(0).deserialize().unwrap();
    println!("Payload b1: {}", payload);
    assert_eq!(payload, 1);
    let payload: i64 = messages.remove(0).deserialize().unwrap();
    println!("Payload b2: {}", payload);
    assert_eq!(payload, 2);

    Ok(())
  }

  #[test_log::test(tokio::test)]
  async fn test_send() -> Result {
    let mut port1 = StringSender {
      port: PortChannel::new("test1"),
    };
    let mut rx = port1.port.open();

    port1.send("first".to_owned())?;

    let message: TransportWrapper = rx.next().await.unwrap().into();
    let payload: String = message.payload.deserialize().unwrap();

    assert_eq!(payload, "first");

    Ok(())
  }

  #[test_log::test(tokio::test)]
  async fn test_done() -> Result {
    let mut port1 = StringSender {
      port: PortChannel::new("test1"),
    };
    let mut rx = port1.port.open();

    port1.done("done".to_owned())?;

    let message: TransportWrapper = rx.next().await.unwrap().into();
    let payload: String = message.payload.deserialize().unwrap();

    assert_eq!(payload, "done");
    let message = rx.next().await.unwrap();
    assert_eq!(message.payload, Packet::done().into());
    Ok(())
  }

  #[test_log::test(tokio::test)]
  async fn test_exception() -> Result {
    let mut port1 = StringSender {
      port: PortChannel::new("test1"),
    };
    let mut rx = port1.port.open();

    port1.send_exception("exc".to_owned())?;

    let message = rx.next().await.unwrap();

    assert_eq!(message.payload, Packet::exception("exc").into());

    Ok(())
  }

  #[test_log::test(tokio::test)]
  async fn test_done_exception() -> Result {
    let mut port1 = StringSender {
      port: PortChannel::new("test1"),
    };
    let mut rx = port1.port.open();

    port1.done_exception("exc".to_owned())?;

    let message = rx.next().await.unwrap();

    assert_eq!(message.payload, Packet::exception("exc").into());
    let message = rx.next().await.unwrap();
    assert_eq!(message.payload, Packet::done().into());
    Ok(())
  }
}
