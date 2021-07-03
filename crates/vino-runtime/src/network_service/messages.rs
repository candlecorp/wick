use std::collections::HashMap;

use actix::Message;
use vino_manifest::NetworkDefinition;
use vino_rpc::SchematicSignature;

use crate::dev::prelude::{
  NetworkError,
  *,
};

type Result<T> = std::result::Result<T, NetworkError>;

#[derive(Message, Debug)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
  pub(crate) network_id: String,
  pub(crate) seed: String,
  pub(crate) network: NetworkDefinition,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) allow_latest: bool,
}

#[derive(Message)]
#[rtype(result = "Result<HashMap<String, MessageTransport>>")]
pub(crate) struct Request {
  pub(crate) schematic: String,
  pub(crate) data: HashMap<String, Vec<u8>>,
}

#[derive(Message)]
#[rtype(result = "Result<Vec<SchematicSignature>>")]
pub(crate) struct ListSchematics {}
