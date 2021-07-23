use std::cell::RefCell;
use std::convert::TryFrom;
use std::io::Read;
use std::pin::Pin;
use std::task::Poll;

use actix::dev::MessageResponse;
use data_encoding::HEXUPPER;
use futures::Stream;
use ring::digest::{
  Context,
  Digest,
  SHA256,
};
use serde::{
  Deserialize,
  Serialize,
};
use tokio::sync::mpsc::UnboundedReceiver;
use uuid::Uuid;
use vino_rpc::port::PacketWrapper;
use vino_wascap::{
  Claims,
  KeyPair,
};

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
  pub encoded_claims: String,
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
pub(crate) fn get_uuid() -> String {
  format!("{}", Uuid::new_v4())
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
    let target_url = target.url();
    let payload = msg.into();
    let claims = Claims::<vino_wascap::Invocation>::new(
      issuer.clone(),
      invocation_id.clone(),
      &target_url,
      &origin.url(),
      &invocation_hash(&target_url, &origin.url(), &payload),
    );
    Invocation {
      origin,
      target,
      msg: payload,
      id: invocation_id,
      encoded_claims: claims.encode(hostkey).unwrap(),
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
    // let claims = Claims::<vino_wascap::Invocation>::new(
    //   issuer.clone(),
    //   invocation_id.clone(),
    //   &target_url,
    //   &origin.url(),
    //   &invocation_hash(&target_url, &origin.url(), &payload),
    // );
    Invocation {
      origin,
      target,
      msg: payload,
      id: invocation_id,
      encoded_claims: "".to_owned(), //claims.encode(hostkey).unwrap(),
      network_id: issuer,
      tx_id: tx_id.to_owned(),
    }
  }
}

pub(crate) fn invocation_hash(
  target_url: &str,
  origin_url: &str,
  msg: &MessageTransport,
) -> String {
  use std::io::Write;
  let mut cleanbytes: Vec<u8> = Vec::new();
  cleanbytes.write_all(origin_url.as_bytes()).unwrap();
  cleanbytes.write_all(target_url.as_bytes()).unwrap();
  match msg {
    MessageTransport::MessagePack(bytes) => cleanbytes.write_all(bytes).unwrap(),
    MessageTransport::Exception(string) => cleanbytes.write_all(string.as_bytes()).unwrap(),
    MessageTransport::Error(string) => cleanbytes.write_all(string.as_bytes()).unwrap(),
    MessageTransport::MultiBytes(bytemap) => {
      for (key, val) in bytemap {
        cleanbytes.write_all(key.as_bytes()).unwrap();
        cleanbytes.write_all(val).unwrap();
      }
    }
    MessageTransport::Test(v) => cleanbytes.write_all(v.as_bytes()).unwrap(),
    MessageTransport::Invalid => cleanbytes.write_all(&[0, 0, 0, 0, 0]).unwrap(),
    MessageTransport::OutputMap(map) => {
      for (key, val) in map {
        cleanbytes.write_all(key.as_bytes()).unwrap();
        cleanbytes
          .write_all(invocation_hash(origin_url, target_url, val).as_bytes())
          .unwrap();
      }
    }
    MessageTransport::Signal(signal) => match signal {
      MessageSignal::Close => cleanbytes.write_all(b"1").unwrap(),
      MessageSignal::OpenBracket => cleanbytes.write_all(b"2").unwrap(),
      MessageSignal::CloseBracket => cleanbytes.write_all(b"3").unwrap(),
    },
  }
  let digest = sha256_digest(cleanbytes.as_slice()).unwrap();
  HEXUPPER.encode(digest.as_ref())
}

fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest, VinoError> {
  let mut context = Context::new(&SHA256);
  let mut buffer = [0; 1024];

  loop {
    let count = reader.read(&mut buffer)?;
    if count == 0 {
      break;
    }
    context.update(&buffer[..count]);
  }

  Ok(context.finish())
}
