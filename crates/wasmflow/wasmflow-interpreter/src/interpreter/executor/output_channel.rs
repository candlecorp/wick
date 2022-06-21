use std::fmt::Debug;

use wasmflow_sdk::v1::transport::TransportWrapper;

use super::error::ExecutionError;

type PacketType = TransportWrapper;
type SenderChannel = tokio::sync::mpsc::Sender<PacketType>;
type ReceiverChannel = tokio::sync::mpsc::Receiver<PacketType>;

const CHANNEL_SIZE: usize = 50;

pub(super) struct Sender {
  inner: SenderChannel,
}

impl Sender {
  pub(super) fn new(tx: SenderChannel) -> Self {
    Self { inner: tx }
  }

  pub(super) async fn send(&self, value: PacketType) -> Result<(), tokio::sync::mpsc::error::SendError<PacketType>> {
    self.inner.send(value).await
  }
}
pub(super) struct Receiver {
  inner: ReceiverChannel,
}
impl Receiver {
  pub(super) fn new(rx: ReceiverChannel) -> Self {
    Self { inner: rx }
  }

  pub(super) fn into_stream(self) -> tokio_stream::wrappers::ReceiverStream<PacketType> {
    tokio_stream::wrappers::ReceiverStream::new(self.inner)
  }
}

pub(super) struct OutputChannel {
  input: Sender,
  output: Option<Receiver>,
}

impl Debug for OutputChannel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Channel").finish()
  }
}

impl Default for OutputChannel {
  fn default() -> Self {
    let (sender, receiver) = tokio::sync::mpsc::channel::<TransportWrapper>(CHANNEL_SIZE);
    Self {
      input: Sender::new(sender),
      output: Some(Receiver::new(receiver)),
    }
  }
}

impl OutputChannel {
  pub(super) async fn push(&self, value: PacketType) -> Result<(), ExecutionError> {
    self.input.send(value).await.map_err(|_| ExecutionError::ChannelSend)
  }

  pub(super) fn detach(&mut self) -> Option<Receiver> {
    self.output.take()
  }
}
