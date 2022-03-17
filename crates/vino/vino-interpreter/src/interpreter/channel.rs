use std::time::Duration;

use once_cell::sync::Lazy;
use uuid::Uuid;
use vino_schematic_graph::{ComponentIndex, PortReference};
use vino_transport::TransportWrapper;

pub(crate) use self::error::Error;
use super::executor::error::ExecutionError;
use crate::interpreter::executor::transaction::Transaction;

pub(crate) type EventPayload = InterpreterEvent;

static CHANNEL_SIZE: usize = 50;
static TIMEOUT: Lazy<Duration> = Lazy::new(|| Duration::from_secs(5));

#[derive(Debug)]
pub(crate) struct Responder {}

impl Responder {}

#[derive(Debug)]
pub(crate) enum InterpreterEvent {
  #[allow(unused)]
  Ping(usize),
  TransactionStart(Box<Transaction>),
  TransactionDone(Uuid),
  ComponentReady(ComponentReady),
  PortData(PortData),
  TransactionOutput(PortData),
  Close,
}

impl InterpreterEvent {
  pub(crate) fn name(&self) -> &str {
    match self {
      InterpreterEvent::Ping(_) => "ping",
      InterpreterEvent::TransactionStart(_) => "tx_start",
      InterpreterEvent::TransactionDone(_) => "tx_done",
      InterpreterEvent::ComponentReady(_) => "component_ready",
      InterpreterEvent::PortData(_) => "port_data",
      InterpreterEvent::TransactionOutput(_) => "tx_output",
      InterpreterEvent::Close => "close",
    }
  }
}

#[derive(Debug, Clone)]
pub(crate) struct ComponentReady {
  pub(crate) tx_id: Uuid,
  pub(crate) index: ComponentIndex,
}

impl ComponentReady {
  pub(crate) fn new(tx_id: Uuid, index: ComponentIndex) -> Self {
    Self { tx_id, index }
  }
}

#[derive(Debug, Clone)]
pub(crate) struct PortData {
  pub(crate) port: PortReference,
  pub(crate) tx_id: Uuid,
  pub(crate) transport: TransportWrapper,
}

impl PortData {
  pub(crate) fn new(tx_id: Uuid, port: PortReference, transport: TransportWrapper) -> Self {
    Self { port, tx_id, transport }
  }
  pub(crate) fn move_port<T: AsRef<str>>(mut self, port: PortReference, name: T) -> Self {
    self.transport.port = name.as_ref().to_owned();
    self.port = port;
    self
  }
}

pub(crate) struct InterpreterChannel {
  sender: tokio::sync::mpsc::Sender<EventPayload>,
  receiver: tokio::sync::mpsc::Receiver<EventPayload>,
}

impl Default for InterpreterChannel {
  fn default() -> Self {
    Self::new()
  }
}

impl std::fmt::Debug for InterpreterChannel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("InterpreterChannel()").finish()
  }
}

impl InterpreterChannel {
  pub(crate) fn new() -> Self {
    let (sender, receiver) = tokio::sync::mpsc::channel(CHANNEL_SIZE);
    Self { sender, receiver }
  }

  pub(crate) fn dispatcher(&self) -> InterpreterDispatchChannel {
    InterpreterDispatchChannel::new(self.sender.clone())
  }

  #[instrument(skip(self))]
  pub(crate) async fn accept(&mut self) -> Result<Option<EventPayload>, ExecutionError> {
    tokio::time::timeout(*TIMEOUT, self.receiver.recv())
      .await
      .map_err(|_| ExecutionError::ChannelError(Error::ReceiveTimeout))
  }
}

#[derive(Clone)]
pub struct InterpreterDispatchChannel {
  sender: tokio::sync::mpsc::Sender<EventPayload>,
}

impl std::fmt::Debug for InterpreterDispatchChannel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("InterpreterRequestChannel()").finish()
  }
}

impl InterpreterDispatchChannel {
  fn new(sender: tokio::sync::mpsc::Sender<EventPayload>) -> Self {
    Self { sender }
  }

  #[instrument(name = "channel_dispatch", skip_all, fields(event = event.name()))]
  pub(crate) async fn dispatch(&self, event: InterpreterEvent) -> Result<(), ExecutionError> {
    trace!("sending to interpreter");
    self
      .sender
      .send_timeout(event, *TIMEOUT)
      .await
      .map_err(|_| ExecutionError::ChannelError(Error::SendTimeout))?;
    Ok(())
  }
}

pub mod error {
  #[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
  pub enum Error {
    #[error("Receive failed")]
    Receive,
    #[error("Receive timed out")]
    ReceiveTimeout,
    #[error("Response failed")]
    Response,
    #[error("Request timed out")]
    SendTimeout,
    #[error("Request failed")]
    Request(RequestError),
  }
  #[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
  pub enum RequestError {
    #[error("Bad request")]
    BadRequest,
  }
}

#[cfg(test)]
mod test {

  use super::*;

  #[tokio::test]
  async fn test_channel() -> anyhow::Result<()> {
    let mut channel = InterpreterChannel::new();

    let child1 = channel.dispatcher();
    let child2 = channel.dispatcher();
    let child3 = channel.dispatcher();

    let join_handle = tokio::task::spawn(async move {
      println!("Handling requests");
      let mut num_handled = 0;
      while let Ok(Some(event)) = channel.accept().await {
        num_handled += 1;
        match event {
          InterpreterEvent::Ping(num) => {
            trace!("ping:{}", num);
          }
          InterpreterEvent::Close => {
            break;
          }
          _ => panic!(),
        }
      }
      println!("Done handling requests");
      num_handled
    });

    tokio::spawn(async move {
      let num = 1;
      println!("Child 1 PING({})", num);
      child1.dispatch(InterpreterEvent::Ping(num)).await.unwrap();
    })
    .await?;

    tokio::spawn(async move {
      let num = 2;
      println!("Child 2 PING({})", num);
      child2.dispatch(InterpreterEvent::Ping(num)).await.unwrap();
    })
    .await?;

    let result = child3.dispatch(InterpreterEvent::Close).await;
    println!("{:?}", result);
    let num_handled = join_handle.await?;

    println!("{:?}", num_handled);
    assert_eq!(num_handled, 3);

    Ok(())
  }
}
