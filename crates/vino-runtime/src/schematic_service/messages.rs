use std::collections::HashMap;

use actix::Message;
use serde::{
  Deserialize,
  Serialize,
};
use vino_rpc::SchematicSignature;

use crate::dev::prelude::*;

type Result<T> = std::result::Result<T, SchematicError>;

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<()>")]
pub(crate) struct UpdateProvider {
  pub(crate) model: ProviderModel,
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<()>")]
pub struct ComponentOutput {
  pub port: String,
  pub invocation_id: String,
  pub payload: Packet,
}
#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<InvocationResponse>")]
pub(crate) struct Request {
  pub(crate) tx_id: String,
  pub(crate) schematic: String,
  pub(crate) payload: HashMap<String, Vec<u8>>,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<()>")]
pub(crate) enum TransactionUpdate {
  ReferenceReady(ReferenceReady),
  SchematicOutput(SchematicOutputReceived),
  SchematicDone(String),
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<()>")]
pub(crate) struct ReferenceReady {
  pub(crate) tx_id: String,
  pub(crate) reference: String,
  pub(crate) payload_map: HashMap<String, MessageTransport>,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<()>")]
pub(crate) struct SchematicOutputReceived {
  pub(crate) port: String,
  pub(crate) tx_id: String,
  pub(crate) payload: MessageTransport,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<SchematicSignature>")]
pub(crate) struct GetSignature {}

#[derive(Debug, Clone, Serialize, Deserialize, Message, PartialEq)]
#[rtype(result = "Result<()>")]
pub struct PayloadReceived {
  pub tx_id: String,
  pub origin: PortReference,
  pub target: PortReference,
  pub payload: MessageTransport,
}

#[derive(Message, Clone)]
#[rtype(result = "Result<()>")]
pub(crate) struct ShortCircuit {
  pub(crate) tx_id: String,
  pub(crate) reference: String,
  pub(crate) payload: MessageTransport,
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<()>")]
pub(crate) struct OutputPortReady {
  pub(crate) port: PortReference,
  pub(crate) tx_id: String,
  pub(crate) payload: MessageTransport,
}

#[derive(Message, Debug)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
  pub(crate) schematic: SchematicDefinition,
  pub(crate) network_provider_channel: Option<ProviderChannel>,
  pub(crate) seed: String,
  pub(crate) allow_latest: bool,
  pub(crate) allowed_insecure: Vec<String>,
}
