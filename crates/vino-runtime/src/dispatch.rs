use std::fmt::Display;
use std::io::Read;

use actix::dev::MessageResponse;
use actix::prelude::Message;
use actix::Actor;
use data_encoding::HEXUPPER;
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
use vino_codec::messagepack::serialize;
use vino_component::Output;
use vino_transport::MessageTransport;
use wascap::prelude::{
  Claims,
  KeyPair,
};

use crate::network::{
  NativeOutputReady,
  Network,
  WapcOutputReady,
};
use crate::util::hlreg::HostLocalSystemService;
use crate::{
  Error,
  Result,
};

/// An invocation for a component, port, or schematic
#[derive(Debug, Clone, Default, Serialize, Deserialize, Message, PartialEq)]
#[rtype(result = "InvocationResponse")]
pub struct Invocation {
  pub origin: VinoEntity,
  pub target: VinoEntity,
  pub msg: MessageTransport,
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
    msg: impl Into<MessageTransport>,
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
  Component(ComponentEntity),
  Provider(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentEntity {
  pub id: String,
  pub reference: String,
  pub name: String,
}

impl Display for ComponentEntity {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}/{}", self.reference, self.id)
  }
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
      VinoEntity::Component(e) => format!("{}://component/{}", URL_SCHEME, e.id),
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
      VinoEntity::Component(e) => format!("component:{}", e.id),
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

  pub fn into_component(self) -> Result<ComponentEntity> {
    match self {
      VinoEntity::Component(s) => Ok(s),
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
  trace!(
    "Guest {} invoking {}:{} (id:{})",
    claims.subject,
    namespace,
    port,
    inv_id
  );
  let network = Network::from_hostlocal_registry(&kp.public_key());
  let msg = WapcOutputReady {
    payload: payload.to_vec(),
    port: port.to_string(),
    invocation_id: inv_id.to_string(),
  };
  network.do_send(msg);
  Ok(vec![])
}

pub(crate) fn native_host_callback(
  kp: KeyPair,
  inv_id: &str,
  namespace: &str,
  port: &str,
  payload: Output,
) -> std::result::Result<Vec<u8>, Box<dyn ::std::error::Error + Sync + Send>> {
  trace!(
    "Native component callback [ns:{}] [port:{}] [inv:{}]",
    namespace,
    port,
    inv_id,
  );
  let network = Network::from_hostlocal_registry(&kp.public_key());
  let msg = NativeOutputReady {
    payload,
    port: port.to_string(),
    invocation_id: inv_id.to_string(),
  };
  network.do_send(msg);
  Ok(vec![])
}
