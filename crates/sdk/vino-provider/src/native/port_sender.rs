use log::*;
use serde::Serialize;
use tokio::sync::mpsc::{
  unbounded_channel,
  UnboundedSender,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::{
  StreamExt,
  StreamMap,
};
use vino_packet::v0::Payload as ComponentPayload;
use vino_packet::{
  Packet,
  PacketWrapper,
};
use vino_transport::{
  TransportStream,
  TransportWrapper,
};

use super::error::Error;

type Result = std::result::Result<(), Error>;

/// The native PortSender trait. This trait encapsulates sending messages out of native ports.
pub trait PortSender {
  /// The type of data that the port outputs.
  type PayloadType: Serialize + Send + 'static;

  /// Get the port buffer that the sender can push to.
  fn get_port(&self) -> std::result::Result<&PortChannel, Error>;

  /// Get the port's name.
  fn get_port_name(&self) -> String;

  /// Send a message.
  fn send(&self, data: &Self::PayloadType) -> Result {
    self.push(Packet::V0(ComponentPayload::success(data)))
  }

  /// Send a message then close the port.
  fn done(&self, data: &Self::PayloadType) -> Result {
    self.send(data)?;
    self.send_message(Packet::V0(ComponentPayload::Done))
  }

  /// Send a complete Output message then close the port.
  fn push(&self, output: Packet) -> Result {
    self.get_port()?.send(PacketWrapper {
      payload: output,
      port: self.get_port_name(),
    })?;
    Ok(())
  }

  /// Send a payload.
  fn send_message(&self, packet: Packet) -> Result {
    self.get_port()?.send(PacketWrapper {
      payload: packet,
      port: self.get_port_name(),
    })?;
    Ok(())
  }

  /// Send a payload then close the port.
  fn done_message(&self, packet: Packet) -> Result {
    self.send_message(packet)?;
    self.send_message(Packet::V0(ComponentPayload::Done))
  }

  /// Send an exception.
  fn send_exception(&self, payload: String) -> Result {
    self.get_port()?.send(PacketWrapper {
      payload: Packet::V0(ComponentPayload::Exception(payload)),
      port: self.get_port_name(),
    })?;
    Ok(())
  }

  /// Send an exception then close the port.
  fn done_exception(&self, payload: String) -> Result {
    self.send_exception(payload)?;
    self.send_message(Packet::V0(ComponentPayload::Done))
  }

  /// Signal that a job is finished with the port.
  fn close(&self) -> Result {
    self.send_message(Packet::V0(ComponentPayload::Done))
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
    let incoming = self.incoming.as_ref().ok_or(Error::SendChannelClosed)?;
    incoming.send(msg)?;
    Ok(())
  }

  /// Merge a list of [PortChannel]s into a TransportStream.
  pub fn merge_all(buffer: &mut [&mut PortChannel]) -> TransportStream {
    let (tx, rx) = unbounded_channel::<TransportWrapper>();

    let mut channels = StreamMap::new();
    for channel in buffer {
      channels.insert(channel.name.clone(), channel.open());
    }

    tokio::spawn(async move {
      while let Some((_, msg)) = channels.next().await {
        match tx.send(msg.into()) {
          Ok(_) => {}
          Err(e) => {
            error!("Unexpected error sending to aggregated stream: {}", e);
          }
        };
      }
    });

    TransportStream::new(UnboundedReceiverStream::new(rx))
  }
}

#[cfg(test)]
mod tests {

  use vino_packet::v0;

  use super::*;
  struct StringSender {
    port: PortChannel,
  }
  impl PortSender for StringSender {
    type PayloadType = String;

    fn get_port(&self) -> std::result::Result<&PortChannel, Error> {
      Ok(&self.port)
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  struct I64Sender {
    port: PortChannel,
  }
  impl PortSender for I64Sender {
    type PayloadType = i64;

    fn get_port(&self) -> std::result::Result<&PortChannel, Error> {
      Ok(&self.port)
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[test_env_log::test(tokio::test)]
  async fn test_merge() -> Result {
    // This sets up the ports, sends data on them, then
    // drops the ports, thus closing them.
    let mut aggregated = {
      let mut port1 = StringSender {
        port: PortChannel::new("test1"),
      };
      let mut port2 = I64Sender {
        port: PortChannel::new("test2"),
      };

      let aggregated = PortChannel::merge_all(&mut [&mut port1.port, &mut port2.port]);

      port1.send(&"First".to_owned())?;
      port2.send(&1)?;
      port1.done(&"Second".to_owned())?;
      port2.done(&2)?;

      aggregated
    };
    let mut messages: Vec<TransportWrapper> = aggregated.collect_port("test1").await;
    assert_eq!(messages.len(), 2);
    assert_eq!(aggregated.buffered_size(), (1, 2));
    let payload: String = messages.remove(0).try_into().unwrap();
    println!("Payload a1: {}", payload);
    assert_eq!(payload, "First");
    let payload: String = messages.remove(0).try_into().unwrap();
    println!("Payload a2: {}", payload);
    assert_eq!(payload, "Second");

    let mut messages: Vec<TransportWrapper> = aggregated.collect_port("test2").await;
    assert_eq!(messages.len(), 2);
    assert_eq!(aggregated.buffered_size(), (0, 0));
    let payload: i64 = messages.remove(0).try_into().unwrap();
    println!("Payload b1: {}", payload);
    assert_eq!(payload, 1);
    let payload: i64 = messages.remove(0).try_into().unwrap();
    println!("Payload b2: {}", payload);
    assert_eq!(payload, 2);

    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn test_send() -> Result {
    let mut port1 = StringSender {
      port: PortChannel::new("test1"),
    };
    let mut rx = port1.port.open();

    port1.send(&"first".to_owned())?;

    let message: TransportWrapper = rx.next().await.unwrap().into();
    let payload: String = message.payload.try_into().unwrap();

    assert_eq!(payload, "first");

    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn test_done() -> Result {
    let mut port1 = StringSender {
      port: PortChannel::new("test1"),
    };
    let mut rx = port1.port.open();

    port1.done(&"done".to_owned())?;

    let message: TransportWrapper = rx.next().await.unwrap().into();
    let payload: String = message.payload.try_into().unwrap();

    assert_eq!(payload, "done");
    let message = rx.next().await.unwrap();
    assert_eq!(message.payload, Packet::V0(v0::Payload::Done));
    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn test_exception() -> Result {
    let mut port1 = StringSender {
      port: PortChannel::new("test1"),
    };
    let mut rx = port1.port.open();

    port1.send_exception("exc".to_owned())?;

    let message = rx.next().await.unwrap();

    assert_eq!(
      message.payload,
      Packet::V0(v0::Payload::Exception("exc".to_owned()))
    );

    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn test_done_exception() -> Result {
    let mut port1 = StringSender {
      port: PortChannel::new("test1"),
    };
    let mut rx = port1.port.open();

    port1.done_exception("exc".to_owned())?;

    let message = rx.next().await.unwrap();

    assert_eq!(
      message.payload,
      Packet::V0(v0::Payload::Exception("exc".to_owned()))
    );
    let message = rx.next().await.unwrap();
    assert_eq!(message.payload, Packet::V0(v0::Payload::Done));
    Ok(())
  }
}
