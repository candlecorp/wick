use std::collections::HashMap;
use std::fmt::Display;
use std::io::Read;

use actix::dev::MessageResponse;
use actix::prelude::Message;
use actix::Actor;
use data_encoding::HEXUPPER;
use futures::executor::block_on;
use ring::digest::{
  Context,
  Digest,
  SHA256,
};
use serde::{
  Deserialize,
  Serialize,
};
use uuid::Uuid;
use vino_guest::OutputPayload;
use wascap::prelude::{
  Claims,
  KeyPair,
};

use crate::network::{
  GetReference,
  Network,
  WapcOutputReady,
};
use crate::schematic::OutputReady;
use crate::util::hlreg::HostLocalSystemService;
use crate::{
  serialize,
  Error,
  Result,
};

/// An invocation for a component, port, or schematic
#[derive(Debug, Clone, Default, Serialize, Deserialize, Message, PartialEq)]
#[rtype(result = "InvocationResponse")]
pub struct Invocation {
  pub origin: VinoEntity,
  pub target: VinoEntity,
  pub msg: MessagePayload,
  pub id: String,
  pub tx_id: String,
  pub encoded_claims: String,
  pub host_id: String,
}

impl<A, M> MessageResponse<A, M> for Invocation
where
  A: Actor,
  M: Message<Result = Invocation>,
{
  fn handle(self, _: &mut A::Context, tx: Option<actix::dev::OneshotSender<Self>>) {
    if let Some(tx) = tx {
      if let Err(e) = tx.send(self) {
        error!("send error (call id:{} target:{:?})", &e.id, &e.target);
      }
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[doc(hidden)]
pub struct InvocationResponse {
  pub tx_id: String,
  pub msg: Vec<u8>,
}

impl InvocationResponse {
  /// Creates a successful invocation response. All invocation responses contain the
  /// invocation ID to which they correlate
  pub fn success(tx_id: String, payload: Vec<u8>) -> InvocationResponse {
    InvocationResponse {
      tx_id,
      msg: payload,
    }
  }

  /// Creates an error response
  pub fn error(tx_id: String, msg: String) -> InvocationResponse {
    let payload = serialize(msg).unwrap();
    InvocationResponse {
      msg: payload,
      tx_id,
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
        error!("InvocationResponse can't be sent : {:?}", e.msg);
      }
    }
  }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessagePayload {
  MessagePack(Vec<u8>),
  MultiBytes(HashMap<String, Vec<u8>>),
  Exception(String),
  Error(String),
}

impl Default for MessagePayload {
  fn default() -> Self {
    Self::MessagePack(vec![])
  }
}

impl MessagePayload {
  pub fn into_bytes(self) -> Result<Vec<u8>> {
    match self {
      MessagePayload::MessagePack(bytes) => Ok(bytes),
      _ => Err(Error::PayloadConversionError("Invalid payload".to_string())),
    }
  }
}

impl From<Vec<u8>> for MessagePayload {
  fn from(v: Vec<u8>) -> Self {
    MessagePayload::MessagePack(v)
  }
}

impl From<&Vec<u8>> for MessagePayload {
  fn from(v: &Vec<u8>) -> Self {
    MessagePayload::MessagePack(v.clone())
  }
}

impl From<&[u8]> for MessagePayload {
  fn from(v: &[u8]) -> Self {
    MessagePayload::MessagePack(v.to_vec())
  }
}

impl From<OutputPayload> for MessagePayload {
  fn from(v: OutputPayload) -> Self {
    match v {
      OutputPayload::MessagePack(v) => MessagePayload::MessagePack(v),
      OutputPayload::Exception(v) => MessagePayload::Exception(v),
      OutputPayload::Error(v) => MessagePayload::Error(v),
    }
  }
}

impl Invocation {
  pub fn uuid() -> String {
    format!("{}", Uuid::new_v4())
  }
  /// Creates an invocation with a specific transaction id, to correlate a chain of
  /// invocations.
  pub fn next(
    tx_id: &str,
    hostkey: &KeyPair,
    origin: VinoEntity,
    target: VinoEntity,
    msg: impl Into<MessagePayload>,
  ) -> Invocation {
    let invocation_id = Invocation::uuid();
    let issuer = hostkey.public_key();
    let target_url = target.url();
    let payload = msg.into();
    let claims = Claims::<wascap::prelude::Invocation>::new(
      issuer.to_string(),
      invocation_id.to_string(),
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
      host_id: issuer,
      tx_id: tx_id.to_string(),
    }
  }
}

pub(crate) fn invocation_hash(target_url: &str, origin_url: &str, msg: &MessagePayload) -> String {
  use std::io::Write;
  let mut cleanbytes: Vec<u8> = Vec::new();
  cleanbytes.write_all(origin_url.as_bytes()).unwrap();
  cleanbytes.write_all(target_url.as_bytes()).unwrap();
  match msg {
    MessagePayload::MessagePack(bytes) => cleanbytes.write_all(bytes).unwrap(),
    MessagePayload::Exception(string) => cleanbytes.write_all(string.as_bytes()).unwrap(),
    MessagePayload::Error(string) => cleanbytes.write_all(string.as_bytes()).unwrap(),
    MessagePayload::MultiBytes(bytemap) => {
      for (key, val) in bytemap {
        cleanbytes.write_all(key.as_bytes()).unwrap();
        cleanbytes.write_all(val).unwrap();
      }
    }
  }
  let digest = sha256_digest(cleanbytes.as_slice()).unwrap();
  HEXUPPER.encode(digest.as_ref())
}

fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest> {
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

#[derive(Debug, Clone, Serialize, Deserialize, Message, PartialEq)]
#[rtype(result = "InvocationResponse")]
pub enum VinoEntity {
  Test(String),
  Schematic(String),
  Port(PortEntity),
  Component(String),
  Provider(String),
}

impl Default for VinoEntity {
  fn default() -> Self {
    Self::Test("default".to_string())
  }
}

impl Display for VinoEntity {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.url())
  }
}
pub(crate) const URL_SCHEME: &str = "wasmbus";

impl VinoEntity {
  /// The URL of the entity
  pub fn url(&self) -> String {
    match self {
      VinoEntity::Test(name) => format!("{}://test/{}", URL_SCHEME, name),
      VinoEntity::Schematic(name) => format!("{}://schematic/{}", URL_SCHEME, name),
      VinoEntity::Component(name) => format!("{}://component/{}", URL_SCHEME, name),
      VinoEntity::Provider(name) => format!("{}://provider/{}", URL_SCHEME, name),
      VinoEntity::Port(port) => format!(
        "{}://{}::{}:{}",
        URL_SCHEME, port.schematic, port.name, port.reference
      ),
    }
  }

  /// The unique (public) key of the entity
  pub fn key(&self) -> String {
    match self {
      VinoEntity::Test(name) => format!("test:{}", name),
      VinoEntity::Schematic(name) => format!("schematic:{}", name),
      VinoEntity::Component(name) => format!("component:{}", name),
      VinoEntity::Provider(name) => format!("provider:{}", name),
      VinoEntity::Port(port) => {
        format!("{}::{}:{}", port.schematic, port.reference, port.name)
      }
    }
  }

  pub fn into_provider(self) -> Result<String> {
    match self {
      VinoEntity::Provider(s) => Ok(s),
      _ => Err(Error::ConversionError),
    }
  }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct PortEntity {
  pub schematic: String,
  pub reference: String,
  pub name: String,
}

impl Display for PortEntity {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}::{}[{}]", self.schematic, self.reference, self.name)
  }
}

pub(crate) fn wapc_host_callback(
  kp: KeyPair,
  claims: Claims<wascap::jwt::Actor>,
  inv_id: &str,
  namespace: &str,
  port: &str,
  payload: &[u8],
) -> std::result::Result<Vec<u8>, Box<dyn ::std::error::Error + Sync + Send>> {
  trace!("Guest {} invoking {}:{}", claims.subject, namespace, port);
  let network = Network::from_hostlocal_registry(&kp.public_key());
  let get_ref = network.send(GetReference {
    inv_id: inv_id.to_string(),
  });
  match block_on(async { get_ref.await })? {
    Some((tx_id, schematic, entity)) => match entity {
      VinoEntity::Component(reference) => {
        debug!(
          "Invocation ID {} resolves to tx {} and reference {}",
          inv_id, tx_id, reference
        );
        let msg = WapcOutputReady {
          payload: payload.to_vec(),
          port: PortEntity {
            name: port.to_string(),
            reference,
            schematic,
          },
          tx_id,
        };
        network.do_send(msg);
        Ok(vec![])
      }
      ent => {
        let e = format!("Reference not implemented. {}", ent);
        error!("{}", e);
        Err(e.into())
      }
    },
    None => {
      let e = format!("Can not resolve invocation {}", inv_id);
      error!("{}", e);
      Err(e.into())
    }
  }
}

pub(crate) fn native_host_callback(
  kp: KeyPair,
  inv_id: &str,
  namespace: &str,
  port: &str,
  payload: &OutputPayload,
) -> std::result::Result<Vec<u8>, Box<dyn ::std::error::Error + Sync + Send>> {
  trace!(
    "Native component callback [ns:{}] [port:{}] [inv:{}]",
    namespace,
    port,
    inv_id,
  );
  let network = Network::from_hostlocal_registry(&kp.public_key());

  let payload = match payload {
    OutputPayload::MessagePack(b) => MessagePayload::MessagePack(b.to_vec()),
    OutputPayload::Exception(e) => MessagePayload::Exception(e.to_string()),
    OutputPayload::Error(e) => MessagePayload::Error(e.to_string()),
  };
  let get_ref = network.send(GetReference {
    inv_id: inv_id.to_string(),
  });
  match block_on(async { get_ref.await })? {
    Some((tx_id, schematic, entity)) => match entity {
      VinoEntity::Component(reference) => {
        let port = PortEntity {
          name: port.to_string(),
          reference,
          schematic,
        };
        debug!(
          "Invocation {} resolves to {} for tx {}",
          inv_id, port, tx_id
        );
        let msg = OutputReady {
          port,
          tx_id,
          payload,
        };
        network.do_send(msg);
        Ok(vec![])
      }
      ent => {
        let e = format!("Reference not implemented. {}", ent);
        error!("{}", e);
        Err(e.into())
      }
    },
    None => {
      let e = format!("Can not resolve invocation {}", inv_id);
      error!("{}", e);
      Err(e.into())
    }
  }
}
