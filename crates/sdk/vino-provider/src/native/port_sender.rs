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
  pub fn new(name: String) -> Self {
    Self {
      name,
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
      trace!("Port closing");
    });

    TransportStream::new(rx)
  }
}
