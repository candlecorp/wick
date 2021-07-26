use std::cell::RefCell;
use std::convert::TryFrom;
use std::pin::Pin;
use std::task::Poll;

use actix::dev::MessageResponse;
use futures::Stream;
use serde::{
  Deserialize,
  Serialize,
};
use tokio::sync::mpsc::UnboundedReceiver;
use vino_rpc::port::PacketWrapper;
use vino_wascap::KeyPair;

use crate::dev::prelude::*;
use crate::error::ConversionError;

#[derive(Debug, Clone)]
pub struct OutputPacket {
  pub port: String,
  pub invocation_id: String,
  pub payload: Packet,
}

impl OutputPacket {
  pub fn from_wrapper(wrapper: PacketWrapper, invocation_id: String) -> Self {
    Self {
      port: wrapper.port,
      payload: wrapper.packet,
      invocation_id,
    }
  }
}

/// An invocation for a component, port, or schematic
#[derive(Debug, Clone, Default, Serialize, Deserialize, Message, PartialEq)]
#[rtype(result = "InvocationResponse")]
pub struct Invocation {
  pub origin: Entity,
  pub target: Entity,
  pub msg: MessageTransport,
  pub id: String,
  pub tx_id: String,
  pub network_id: String,
}

impl<A, M> MessageResponse<A, M> for Invocation
where
  A: Actor,
  M: Message<Result = Invocation>,
{
  fn handle(self, _: &mut A::Context, tx: Option<actix::dev::OneshotSender<Self>>) {
    if let Some(tx) = tx {
      if let Err(e) = tx.send(self) {
        error!("Send error (call id:{} target:{:?})", &e.id, &e.target);
      }
    }
  }
}

impl TryFrom<Invocation> for vino_rpc::rpc::Invocation {
  type Error = VinoError;
  fn try_from(inv: Invocation) -> Result<Self, VinoError> {
    Ok(vino_rpc::rpc::Invocation {
      origin: inv.origin.url(),
      target: inv.target.url(),
      msg: inv.msg.into_multibytes()?,
      id: inv.id,
      network_id: inv.network_id,
    })
  }
}

#[derive(Debug)]
pub enum InvocationResponse {
  Stream { tx_id: String, rx: ResponseStream },
  Error { tx_id: String, msg: String },
}

#[derive(Debug)]
pub struct ResponseStream {
  rx: RefCell<UnboundedReceiver<OutputPacket>>,
}

impl Stream for ResponseStream {
  type Item = OutputPacket;

  fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
    let mut rx = self.rx.borrow_mut();
    match rx.poll_recv(cx) {
      Poll::Ready(opt) => match opt {
        Some(output) => {
          if output.payload == Packet::V0(packet::v0::Payload::Close) {
            rx.close();
            Poll::Ready(None)
          } else {
            Poll::Ready(Some(output))
          }
        }
        None => Poll::Ready(None),
      },
      Poll::Pending => Poll::Pending,
    }
  }
}

impl ResponseStream {
  #[must_use]
  pub fn new(rx: UnboundedReceiver<OutputPacket>) -> Self {
    Self {
      rx: RefCell::new(rx),
    }
  }
}

pub(crate) fn inv_error(tx_id: &str, msg: &str) -> InvocationResponse {
  InvocationResponse::error(tx_id.to_owned(), msg.to_owned())
}

impl InvocationResponse {
  /// Creates a successful invocation response stream. Response include the receiving end
  /// of an unbounded channel to listen for future output.
  #[must_use]
  pub fn stream(tx_id: String, rx: UnboundedReceiver<OutputPacket>) -> InvocationResponse {
    trace!("Creating stream");
    InvocationResponse::Stream {
      tx_id,
      rx: ResponseStream::new(rx),
    }
  }

  /// Creates an error response
  #[must_use]
  pub fn error(tx_id: String, msg: String) -> InvocationResponse {
    InvocationResponse::Error { tx_id, msg }
  }

  pub fn tx_id(&self) -> &str {
    match self {
      InvocationResponse::Stream { tx_id, .. } => tx_id,
      InvocationResponse::Error { tx_id, .. } => tx_id,
    }
  }

  pub fn to_stream(self) -> Result<(String, ResponseStream), ConversionError> {
    match self {
      InvocationResponse::Stream { tx_id, rx } => Ok((tx_id, rx)),
      _ => Err(ConversionError("InvocationResponse to stream")),
    }
  }

  pub fn to_error(self) -> Result<(String, String), ConversionError> {
    match self {
      InvocationResponse::Error { tx_id, msg } => Ok((tx_id, msg)),
      _ => Err(ConversionError("InvocationResponse to error")),
    }
  }
}

impl<A, M> MessageResponse<A, M> for InvocationResponse
where
  A: Actor,
  M: Message<Result = InvocationResponse>,
{
  fn handle(self, _: &mut A::Context, tx: Option<actix::dev::OneshotSender<Self>>) {
    if let Some(tx) = tx {
      if let Err(e) = tx.send(self) {
        error!("InvocationResponse can't be sent for tx_id {}", e.tx_id());
      }
    }
  }
}
impl Invocation {
  /// Creates an invocation with a new transaction id
  pub fn new(
    hostkey: &KeyPair,
    origin: Entity,
    target: Entity,
    msg: impl Into<MessageTransport>,
  ) -> Invocation {
    let tx_id = get_uuid();
    let invocation_id = get_uuid();
    let issuer = hostkey.public_key();
    let payload = msg.into();
    Invocation {
      origin,
      target,
      msg: payload,
      id: invocation_id,
      network_id: issuer,
      tx_id,
    }
  }
  /// Creates an invocation with a specific transaction id, to correlate a chain of
  /// invocations.
  pub fn next(
    tx_id: &str,
    hostkey: &KeyPair,
    origin: Entity,
    target: Entity,
    msg: impl Into<MessageTransport>,
  ) -> Invocation {
    let invocation_id = get_uuid();
    let issuer = hostkey.public_key();
    // let target_url = target.url();
    let payload = msg.into();
    Invocation {
      origin,
      target,
      msg: payload,
      id: invocation_id,
      network_id: issuer,
      tx_id: tx_id.to_owned(),
    }
  }
}
