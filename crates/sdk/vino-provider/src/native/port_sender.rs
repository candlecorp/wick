use log::*;
use serde::Serialize;
use tokio::sync::mpsc::{
  unbounded_channel,
  UnboundedReceiver,
  UnboundedSender,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::{
  StreamExt,
  StreamMap,
};
use vino_component::v0::Payload as ComponentPayload;
use vino_component::{
  Packet,
  PacketWrapper,
};
use vino_transport::{
  MessageTransportStream,
  TransportWrapper,
};

use super::error::Error;

type Result = std::result::Result<(), Error>;

/// The native PortSender trait. This trait encapsulates sending messages out of native ports.
pub trait PortSender {
  /// The type of data that the port outputs.
  type PayloadType: Serialize + Send + 'static;

  /// Get the port buffer that the sender can push to.
  fn get_port(&self) -> UnboundedSender<PacketWrapper>;

  /// Get the port's name.
  fn get_port_name(&self) -> String;

  /// Buffer a message.
  fn send(&self, data: &Self::PayloadType) -> Result {
    self.push(Packet::V0(ComponentPayload::success(data)))
  }

  /// Buffer a message then close the port.
  fn done(&self, data: &Self::PayloadType) -> Result {
    self.send(data)?;
    self.send_message(Packet::V0(ComponentPayload::Done))
  }

  /// Buffer a complete Output message then close the port.
  fn push(&self, output: Packet) -> Result {
    self.get_port().send(PacketWrapper {
      payload: output,
      port: self.get_port_name(),
    })?;
    Ok(())
  }

  /// Buffer a payload.
  fn send_message(&self, packet: Packet) -> Result {
    self.get_port().send(PacketWrapper {
      payload: packet,
      port: self.get_port_name(),
    })?;
    Ok(())
  }

  /// Buffer a payload then close the port.
  fn done_message(&self, packet: Packet) -> Result {
    self.send_message(packet)?;
    self.send_message(Packet::V0(ComponentPayload::Done))
  }

  /// Buffer an exception.
  fn send_exception(&self, payload: String) -> Result {
    self.get_port().send(PacketWrapper {
      payload: Packet::V0(ComponentPayload::Exception(payload)),
      port: self.get_port_name(),
    })?;
    Ok(())
  }

  /// Buffer an exception then close the port.
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
  /// An [UnboundedSender<PacketWrapper>] after initialized.
  pub channel: Option<UnboundedSender<PacketWrapper>>,
}

impl PortChannel {
  /// Constructor for a [PortChannel].
  pub fn new(name: String) -> Self {
    Self {
      name,
      channel: None,
    }
  }

  /// Initialize the [PortChannel] and return a receiver.
  pub fn init(&mut self) -> UnboundedReceiver<PacketWrapper> {
    let (tx, rx) = unbounded_channel();
    self.channel = Some(tx);
    rx
  }

  /// Merge a list of [PortChannel]s into a MessageTransportStream
  pub fn merge_all(buffer: &mut [&mut PortChannel]) -> MessageTransportStream {
    let (tx, rx) = unbounded_channel::<TransportWrapper>();

    let mut map = StreamMap::new();
    for channel in buffer {
      map.insert(
        channel.name.clone(),
        UnboundedReceiverStream::new(channel.init()),
      );
    }

    tokio::spawn(async move {
      while let Some((_, v)) = map.next().await {
        let transport: TransportWrapper = v.into();

        if let Err(e) = tx.send(transport) {
          error!("Internal error sending to aggregated output stream {}", e);
        }
      }
    });
    MessageTransportStream::new(rx)
  }
}
