use std::collections::HashMap;
use std::fmt::Display;
use std::io::Read;

use actix::dev::MessageResponse;
use actix::prelude::Message;
use actix::Actor;

use crate::serialize;
use crate::Result;
use data_encoding::HEXUPPER;
use ring::digest::Context;
use ring::digest::Digest;
use ring::digest::SHA256;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wascap::prelude::{Claims, KeyPair};

use super::port_entity::PortEntity;
/// An immutable representation of an invocation within wasmcloud
#[derive(Debug, Clone, Serialize, Deserialize, Message, PartialEq)]
#[rtype(result = "InvocationResponse")]
pub struct Invocation {
    pub origin: VinoEntity,
    pub target: VinoEntity,
    pub operation: String,
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
                error!(
                    "send error (call id:{} target:{:?} op:{})",
                    &e.id, &e.target, &e.operation
                );
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
        let payload = serialize(msg).unwrap_or_else(|_| vec![1, 1, 1, 1]);
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
            if tx.send(self).is_err() {} // TODO
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessagePayload {
    Bytes(Vec<u8>),
    MultiBytes(HashMap<String, Vec<u8>>),
    Exception(String),
    Error(String),
}

impl MessagePayload {
    pub fn get_bytes(self) -> Result<Vec<u8>> {
        match self {
            MessagePayload::Bytes(bytes) => Ok(bytes),
            _ => Err(anyhow!("Invalid payload").into()),
        }
    }
}

impl From<Vec<u8>> for MessagePayload {
    fn from(v: Vec<u8>) -> Self {
        MessagePayload::Bytes(v)
    }
}

impl From<&Vec<u8>> for MessagePayload {
    fn from(v: &Vec<u8>) -> Self {
        MessagePayload::Bytes(v.clone())
    }
}
impl From<&[u8]> for MessagePayload {
    fn from(v: &[u8]) -> Self {
        MessagePayload::Bytes(v.to_vec())
    }
}

impl Invocation {
    /// Creates an invocation with a specific transaction id, to correlate a chain of
    /// invocations.
    pub fn next(
        tx_id: &str,
        hostkey: &KeyPair,
        origin: VinoEntity,
        target: VinoEntity,
        op: &str,
        msg: impl Into<MessagePayload>,
    ) -> Invocation {
        let subject = format!("{}", Uuid::new_v4());
        let issuer = hostkey.public_key();
        let target_url = format!("{}/{}", target.url(), op);
        let payload = msg.into();
        let claims = Claims::<wascap::prelude::Invocation>::new(
            issuer.to_string(),
            subject.to_string(),
            &target_url,
            &origin.url(),
            &invocation_hash(&target_url, &origin.url(), &payload),
        );
        Invocation {
            origin,
            target,
            operation: op.to_string(),
            msg: payload,
            id: subject,
            encoded_claims: claims.encode(&hostkey).unwrap(),
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
        MessagePayload::Bytes(bytes) => cleanbytes.write_all(&bytes).unwrap(),
        MessagePayload::Exception(string) => cleanbytes.write_all(&string.as_bytes()).unwrap(),
        MessagePayload::Error(string) => cleanbytes.write_all(&string.as_bytes()).unwrap(),
        MessagePayload::MultiBytes(bytemap) => {
            for (key, val) in bytemap {
                cleanbytes.write_all(key.as_bytes()).unwrap();
                cleanbytes.write_all(&val).unwrap();
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
    Schematic(String),
    Port(PortEntity),
    Component(String),
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
            VinoEntity::Schematic(name) => format!("{}://schematic/{}", URL_SCHEME, name),
            VinoEntity::Component(name) => format!("{}://component/{}", URL_SCHEME, name),
            VinoEntity::Port(port) => {
                format!(
                    "{}://namespace/{}/port/{}/in/{}/{}",
                    URL_SCHEME, port.schematic, port.parent, port.name, port.reference
                )
            }
        }
    }

    /// The unique (public) key of the entity
    pub fn key(&self) -> String {
        match self {
            VinoEntity::Schematic(name) => format!("schematic:{}", name),
            VinoEntity::Component(name) => format!("component:{}", name),
            VinoEntity::Port(port) => {
                format!("{}:in:{}:{}", port.parent, port.name, port.reference)
            }
        }
    }
}

pub(crate) fn wapc_host_callback(
    _kp: KeyPair,
    claims: Claims<wascap::jwt::Actor>,
    _link_name: &str,
    namespace: &str,
    operation: &str,
    _payload: &[u8],
) -> std::result::Result<Vec<u8>, Box<dyn ::std::error::Error + Sync + Send>> {
    trace!(
        "Guest {} invoking {}:{}",
        claims.subject,
        namespace,
        operation
    );
    Ok(vec![])
}
