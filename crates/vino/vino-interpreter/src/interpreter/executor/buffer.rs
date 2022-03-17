use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::Mutex;
use vino_transport::TransportWrapper;

use super::error::ExecutionError;

type SenderChannel = tokio::sync::mpsc::Sender<PacketType>;
type ReceiverChannel = tokio::sync::mpsc::Receiver<PacketType>;

pub(super) struct Sender {
  sent: AtomicUsize,
  inner: SenderChannel,
}

impl Sender {
  pub(super) fn new(tx: SenderChannel) -> Self {
    Self {
      sent: AtomicUsize::new(0),
      inner: tx,
    }
  }
  pub(super) fn sent(&self) -> usize {
    self.sent.fetch_add(0, Ordering::Relaxed)
  }
  pub(super) async fn send(&self, value: PacketType) -> Result<(), tokio::sync::mpsc::error::SendError<PacketType>> {
    self.inner.send(value).await
  }
}
pub(super) struct Receiver {
  taken: usize,
  inner: Mutex<ReceiverChannel>,
}
impl Receiver {
  pub(super) fn new(rx: ReceiverChannel) -> Self {
    Self {
      taken: 0,
      inner: Mutex::new(rx),
    }
  }
  pub(super) async fn recv(&self) -> Option<PacketType> {
    let mut lock = self.inner.lock().await;
    lock.recv().await
  }
  pub(super) fn into_stream(self) -> tokio_stream::wrappers::ReceiverStream<PacketType> {
    let inner = self.inner.into_inner();
    tokio_stream::wrappers::ReceiverStream::new(inner)
  }
}

type PacketType = TransportWrapper;

const CHANNEL_SIZE: usize = 50;

pub(super) struct PacketBuffer {
  input: Sender,
  output: Option<Receiver>,
}

impl Debug for PacketBuffer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Channel").finish()
  }
}

impl Default for PacketBuffer {
  fn default() -> Self {
    let (sender, receiver) = tokio::sync::mpsc::channel::<TransportWrapper>(CHANNEL_SIZE);
    Self {
      input: Sender::new(sender),
      output: Some(Receiver::new(receiver)),
    }
  }
}

impl PacketBuffer {
  pub(super) async fn push(&self, value: PacketType) -> Result<(), ExecutionError> {
    self.input.sent.fetch_add(1, Ordering::SeqCst);

    self.input.send(value).await.map_err(|_| ExecutionError::ChannelSend)
  }

  pub(super) fn has_data(&self) -> Result<bool, ExecutionError> {
    let sent = self.input.sent();
    match &self.output {
      Some(rx) => Ok(rx.taken != sent),
      None => Err(ExecutionError::ChannelTaken),
    }
  }

  pub(super) async fn receive(&self) -> Result<Option<TransportWrapper>, ExecutionError> {
    match &self.output {
      Some(output) => {
        let result = output.recv().await;
        Ok(result)
      }
      None => Err(ExecutionError::ChannelTaken),
    }
  }

  pub(super) fn detach(&mut self) -> Option<Receiver> {
    self.output.take()
  }
}
