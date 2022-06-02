use uuid::Uuid;
use wasmflow_schematic_graph::{ComponentIndex, PortReference};
use wasmflow_transport::MessageTransport;
use wasmflow_invocation::Invocation;

pub(crate) use self::error::Error;
use super::executor::error::ExecutionError;
use crate::interpreter::executor::transaction::Transaction;

static CHANNEL_SIZE: usize = 50;

const CHANNEL_UUID: Uuid = Uuid::from_bytes([
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF,
]);

#[derive(Debug)]
pub struct Event {
  pub(crate) tx_id: Uuid,
  pub(crate) kind: EventKind,
}

impl Event {
  pub(crate) fn new(tx_id: Uuid, kind: EventKind) -> Self {
    Self { tx_id, kind }
  }

  #[must_use]
  pub fn tx_id(&self) -> &Uuid {
    &self.tx_id
  }

  #[must_use]
  pub fn name(&self) -> &str {
    self.kind.name()
  }

  pub fn kind(&self) -> &EventKind {
    &self.kind
  }

  // constructors
  pub(crate) fn close(error: Option<ExecutionError>) -> Self {
    Event::new(CHANNEL_UUID, EventKind::Close(error))
  }

  pub(crate) fn call_complete(tx_id: Uuid, component_index: ComponentIndex) -> Self {
    Event::new(tx_id, EventKind::CallComplete(CallComplete::new(component_index)))
  }

  pub(crate) fn invocation(index: ComponentIndex, invocation: Invocation) -> Self {
    Event::new(invocation.tx_id, EventKind::Invocation(index, Box::new(invocation)))
  }

  pub(crate) fn port_status_change(tx_id: Uuid, port_ref: PortReference) -> Self {
    Event::new(tx_id, EventKind::PortStatusChange(port_ref))
  }

  pub(crate) fn call_err(tx_id: Uuid, component_index: ComponentIndex, err: MessageTransport) -> Self {
    Event::new(
      tx_id,
      EventKind::CallComplete(CallComplete {
        index: component_index,
        err: Some(err),
      }),
    )
  }

  pub(crate) fn port_data(tx_id: Uuid, port: PortReference) -> Self {
    Event::new(tx_id, EventKind::PortData(port))
  }

  pub(crate) fn tx_done(tx_id: Uuid) -> Self {
    Event::new(tx_id, EventKind::TransactionDone)
  }

  pub(crate) fn tx_start(tx: Box<Transaction>) -> Self {
    Event::new(tx.id(), EventKind::TransactionStart(tx))
  }
}

#[derive(Debug)]
#[must_use]
pub enum EventKind {
  #[allow(unused)]
  Ping(usize),
  TransactionStart(Box<Transaction>),
  TransactionDone,
  PortData(PortReference),
  PortStatusChange(PortReference),
  Invocation(ComponentIndex, Box<Invocation>),
  CallComplete(CallComplete),
  Close(Option<ExecutionError>),
}

impl EventKind {
  pub(crate) fn name(&self) -> &str {
    match self {
      EventKind::Ping(_) => "ping",
      EventKind::TransactionStart(_) => "tx_start",
      EventKind::TransactionDone => "tx_done",
      EventKind::PortStatusChange(_) => "port_status_change",
      EventKind::PortData(_) => "port_data",
      EventKind::Invocation(_, _) => "invocation",
      EventKind::CallComplete(_) => "call_complete",
      EventKind::Close(_) => "close",
    }
  }
}

#[derive(Debug, Clone)]
pub struct CallComplete {
  pub(crate) index: ComponentIndex,
  pub(crate) err: Option<MessageTransport>,
}

impl CallComplete {
  fn new(component_index: ComponentIndex) -> Self {
    Self {
      index: component_index,
      err: None,
    }
  }
  pub fn index(&self) -> ComponentIndex {
    self.index
  }
  pub fn err(&self) -> &Option<MessageTransport> {
    &self.err
  }
}

pub(crate) struct InterpreterChannel {
  sender: tokio::sync::mpsc::Sender<Event>,
  receiver: tokio::sync::mpsc::Receiver<Event>,
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

  pub(crate) async fn accept(&mut self) -> Option<Event> {
    self.receiver.recv().await
  }
}

#[derive(Clone)]
pub struct InterpreterDispatchChannel {
  sender: tokio::sync::mpsc::Sender<Event>,
}

impl std::fmt::Debug for InterpreterDispatchChannel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("InterpreterRequestChannel()").finish()
  }
}

impl InterpreterDispatchChannel {
  fn new(sender: tokio::sync::mpsc::Sender<Event>) -> Self {
    Self { sender }
  }

  pub(crate) async fn dispatch(&self, event: Event) -> Result<(), ExecutionError> {
    trace!(event = event.name(), "dispatching event");
    self
      .sender
      .send(event)
      .await
      .map_err(|_| ExecutionError::ChannelError(Error::Send))?;
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
    #[error("Send failed")]
    Send,
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
      while let Some(event) = channel.accept().await {
        num_handled += 1;
        match event.kind {
          EventKind::Ping(num) => {
            trace!("ping:{}", num);
          }
          EventKind::Close(_) => {
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
      child1
        .dispatch(Event::new(Uuid::new_v4(), EventKind::Ping(num)))
        .await
        .unwrap();
    })
    .await?;

    tokio::spawn(async move {
      let num = 2;
      println!("Child 2 PING({})", num);
      child2
        .dispatch(Event::new(Uuid::new_v4(), EventKind::Ping(num)))
        .await
        .unwrap();
    })
    .await?;

    let result = child3.dispatch(Event::close(None)).await;
    println!("{:?}", result);
    let num_handled = join_handle.await?;

    println!("{:?}", num_handled);
    assert_eq!(num_handled, 3);

    Ok(())
  }
}
