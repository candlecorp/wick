use flow_graph::{NodeIndex, PortReference};
use tracing::Span;
use uuid::Uuid;
use wick_packet::{Invocation, PacketPayload};

pub(crate) use self::error::Error;
use super::executor::error::ExecutionError;
use crate::interpreter::executor::context::ExecutionContext;

static CHANNEL_SIZE: usize = 50;

const CHANNEL_UUID: Uuid = Uuid::from_bytes([
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF,
]);

#[derive(Debug)]
pub struct Event {
  pub(crate) ctx_id: Uuid,
  pub(crate) kind: EventKind,
  pub(crate) span: Option<Span>,
}

impl Event {
  pub(crate) fn new(ctx_id: Uuid, kind: EventKind, span: Option<Span>) -> Self {
    Self { ctx_id, kind, span }
  }

  #[must_use]
  pub fn ctx_id(&self) -> &Uuid {
    &self.ctx_id
  }

  #[must_use]
  pub fn name(&self) -> &str {
    self.kind.name()
  }

  pub fn kind(&self) -> &EventKind {
    &self.kind
  }
}

#[derive(Debug)]
#[must_use]
pub enum EventKind {
  Ping(usize),
  ExecutionStart(Box<ExecutionContext>),
  ExecutionDone,
  PortData(PortReference),
  Invocation(NodeIndex, Box<Invocation>),
  CallComplete(CallComplete),
  Close(Option<ExecutionError>),
}

impl EventKind {
  pub(crate) fn name(&self) -> &str {
    match self {
      EventKind::Ping(_) => "ping",
      EventKind::ExecutionStart(_) => "exec_start",
      EventKind::ExecutionDone => "exec_done",
      EventKind::PortData(_) => "port_data",
      EventKind::Invocation(_, _) => "invocation",
      EventKind::CallComplete(_) => "call_complete",
      EventKind::Close(_) => "close",
    }
  }
}

#[derive(Debug, Clone)]
pub struct CallComplete {
  pub(crate) index: NodeIndex,
  pub(crate) err: Option<PacketPayload>,
}

impl CallComplete {
  fn new(component_index: NodeIndex) -> Self {
    Self {
      index: component_index,
      err: None,
    }
  }
  pub fn index(&self) -> NodeIndex {
    self.index
  }
  pub fn err(&self) -> &Option<PacketPayload> {
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

  pub(crate) fn dispatcher(&self, span: Option<Span>) -> InterpreterDispatchChannel {
    InterpreterDispatchChannel::new(self.sender.clone(), span)
  }

  pub(crate) async fn accept(&mut self) -> Option<Event> {
    self.receiver.recv().await
  }
}

#[derive(Clone)]
pub(crate) struct InterpreterDispatchChannel {
  span: Option<Span>,
  sender: tokio::sync::mpsc::Sender<Event>,
}

impl std::fmt::Debug for InterpreterDispatchChannel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("InterpreterRequestChannel()").finish()
  }
}

impl InterpreterDispatchChannel {
  fn new(sender: tokio::sync::mpsc::Sender<Event>, span: Option<Span>) -> Self {
    Self { sender, span }
  }

  pub(crate) fn with_span(self, span: Span) -> Self {
    Self {
      sender: self.sender,
      span: Some(span),
    }
  }

  pub(crate) fn dispatch(&self, event: Event) {
    let tx = self.sender.clone();

    tokio::task::spawn(async move {
      if tx.send(event).await.is_err() {
        warn!("interpreter channel closed unexpectedly. This is likely due to an intentional shutdown while there are still events processing.");
      }
    });
  }

  pub(crate) fn dispatch_done(&self, ctx_id: Uuid) {
    self.dispatch(Event::new(ctx_id, EventKind::ExecutionDone, self.span.clone()));
  }

  pub(crate) fn dispatch_data(&self, ctx_id: Uuid, port: PortReference) {
    self.dispatch(Event::new(ctx_id, EventKind::PortData(port), self.span.clone()));
  }

  pub(crate) fn dispatch_close(&self, error: Option<ExecutionError>) {
    self.dispatch(Event::new(CHANNEL_UUID, EventKind::Close(error), self.span.clone()));
  }

  pub(crate) fn dispatch_start(&self, ctx: Box<ExecutionContext>) {
    self.dispatch(Event::new(ctx.id(), EventKind::ExecutionStart(ctx), self.span.clone()));
  }

  pub(crate) fn dispatch_call_complete(&self, ctx_id: Uuid, op_index: usize) {
    self.dispatch(Event::new(
      ctx_id,
      EventKind::CallComplete(CallComplete::new(op_index)),
      self.span.clone(),
    ));
  }

  pub(crate) fn dispatch_op_err(&self, ctx_id: Uuid, op_index: usize, signal: PacketPayload) {
    self.dispatch(Event::new(
      ctx_id,
      EventKind::CallComplete(CallComplete {
        index: op_index,
        err: Some(signal),
      }),
      self.span.clone(),
    ));
  }
}

pub(crate) mod error {
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

    let child1 = channel.dispatcher(None);
    let child2 = channel.dispatcher(None);
    let child3 = channel.dispatcher(None);

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
      child1.dispatch(Event::new(Uuid::new_v4(), EventKind::Ping(num), None));
    })
    .await?;

    tokio::spawn(async move {
      let num = 2;
      println!("Child 2 PING({})", num);
      child2.dispatch(Event::new(Uuid::new_v4(), EventKind::Ping(num), None));
    })
    .await?;

    child3.dispatch_close(None);
    let num_handled = join_handle.await?;

    println!("{:?}", num_handled);
    assert_eq!(num_handled, 3);

    Ok(())
  }
}
