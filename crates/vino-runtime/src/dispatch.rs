use std::convert::TryFrom;

use actix::dev::MessageResponse;
use serde::{
  Deserialize,
  Serialize,
};
use tokio::sync::mpsc::UnboundedReceiver;
use vino_rpc::convert_transport_map;
use vino_transport::message_transport::TransportMap;
use vino_wascap::KeyPair;

use crate::dev::prelude::*;

/// An invocation for a component, port, or schematic
#[derive(Debug, Clone, Default, Serialize, Deserialize, Message)]
#[rtype(result = "InvocationResponse")]
pub struct Invocation {
  pub origin: Entity,
  pub target: Entity,
  pub msg: TransportMap,
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
  type Error = RuntimeError;
  fn try_from(inv: Invocation) -> Result<Self, RuntimeError> {
    Ok(vino_rpc::rpc::Invocation {
      origin: inv.origin.url(),
      target: inv.target.url(),
      msg: convert_transport_map(inv.msg),
      id: inv.id,
      network_id: inv.network_id,
    })
  }
}

#[derive(Debug)]
pub enum InvocationResponse {
  Stream {
    tx_id: String,
    rx: MessageTransportStream,
  },
  Error {
    tx_id: String,
    msg: String,
  },
}

pub(crate) fn inv_error(tx_id: &str, msg: &str) -> InvocationResponse {
  InvocationResponse::error(tx_id.to_owned(), msg.to_owned())
}

impl InvocationResponse {
  /// Creates a successful invocation response stream. Response include the receiving end
  /// of an unbounded channel to listen for future output.
  #[must_use]
  pub fn stream(tx_id: String, rx: UnboundedReceiver<TransportWrapper>) -> InvocationResponse {
    InvocationResponse::Stream {
      tx_id,
      rx: MessageTransportStream::new(rx),
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

  pub fn ok(self) -> Result<MessageTransportStream, InvocationError> {
    match self {
      InvocationResponse::Stream { rx, .. } => Ok(rx),
      InvocationResponse::Error { msg, .. } => Err(InvocationError(msg)),
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
  #[must_use]
  pub fn new(hostkey: &KeyPair, origin: Entity, target: Entity, msg: TransportMap) -> Invocation {
    let tx_id = get_uuid();
    let invocation_id = get_uuid();
    let issuer = hostkey.public_key();

    Invocation {
      origin,
      target,
      msg,
      id: invocation_id,
      network_id: issuer,
      tx_id,
    }
  }
  /// Creates an invocation with a specific transaction id, to correlate a chain of
  /// invocations.
  #[must_use]
  pub fn next(
    tx_id: &str,
    hostkey: &KeyPair,
    origin: Entity,
    target: Entity,
    msg: TransportMap,
  ) -> Invocation {
    let invocation_id = get_uuid();
    let issuer = hostkey.public_key();
    Invocation {
      origin,
      target,
      msg,
      id: invocation_id,
      network_id: issuer,
      tx_id: tx_id.to_owned(),
    }
  }
}
