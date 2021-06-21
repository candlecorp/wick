use std::collections::HashMap;
use std::pin::Pin;
use std::task::Poll;

use actix::prelude::*;
use futures::Future;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use vino_transport::serialize;

use crate::{
  Error,
  InvocationResponse,
  MessagePayload,
  Result,
};

type SchematicOutput = HashMap<String, Option<MessagePayload>>;
type TransactionOutputs = HashMap<(String, String), SchematicOutput>;

static SCHEMATIC_RESPONSES: Lazy<Mutex<TransactionOutputs>> =
  Lazy::new(|| Mutex::new(TransactionOutputs::new()));

pub(crate) fn is_response_ready(tx_id: &str, schematic_name: &str) -> Result<bool> {
  let mut responses = SCHEMATIC_RESPONSES.lock();

  let outputs = responses
    .get_mut(&(tx_id.to_string(), schematic_name.to_string()))
    .ok_or_else(|| Error::SchematicError(format!("Transaction {} not found", tx_id)))?;

  let mut ready = true;

  for val in outputs.values() {
    ready = ready && val.is_some();
  }
  Ok(ready)
}

pub(crate) fn push_to_schematic_output(
  tx_id: &str,
  schematic: &str,
  port: &str,
  data: MessagePayload,
) -> Result<()> {
  trace!("Pushing output {}[{}] for tx: {}", schematic, port, tx_id);
  trace!("{:?}", data);
  let mut responses = SCHEMATIC_RESPONSES.lock();

  let outputs = responses
    .get_mut(&(tx_id.to_string(), schematic.to_string()))
    .ok_or_else(|| {
      Error::SchematicError(format!(
        "Schematic '{}' for transaction '{}' not initialized",
        schematic, tx_id
      ))
    })?;
  outputs.insert(port.to_string(), Some(data));
  Ok(())
}

pub(crate) fn initialize_schematic_output(tx_id: &str, schematic: &str, ports: Vec<String>) {
  trace!(
    "Initializing schematic output for '{}' on tx: {} ",
    schematic,
    tx_id
  );
  let mut outputs = SchematicOutput::new();
  for port in ports {
    outputs.insert(port.to_string(), None);
  }
  let mut responses = SCHEMATIC_RESPONSES.lock();

  responses.insert((tx_id.to_string(), schematic.to_string()), outputs);
}

pub(crate) fn get_schematic_output(tx_id: &str, schematic: &str) -> Result<SchematicResponse> {
  let responses = SCHEMATIC_RESPONSES.lock();

  match responses.get(&(tx_id.to_string(), schematic.to_string())) {
    Some(_) => Ok(SchematicResponse::new(tx_id, tx_id, schematic)),
    None => Err("Schematic not initialized".into()),
  }
}

#[derive(Debug)]
pub(crate) struct SchematicResponse {
  name: String,
  tx_id: String,
  inv_id: String,
}

impl SchematicResponse {
  pub(crate) fn new(tx_id: &str, inv_id: &str, name: &str) -> Self {
    SchematicResponse {
      tx_id: tx_id.to_string(),
      inv_id: inv_id.to_string(),
      name: name.to_string(),
    }
  }
}

impl Actor for SchematicResponse {
  type Context = Context<Self>;
}

impl Future for SchematicResponse {
  type Output = InvocationResponse;

  fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<InvocationResponse> {
    let ready = is_response_ready(&self.tx_id, &self.name);

    let mut responses = SCHEMATIC_RESPONSES.lock();
    let tx_id = self.tx_id.to_string();

    match ready {
      Ok(ready) => {
        if ready {
          trace!("Schematic '{}' on tx '{}' is ready", self.name, self.tx_id);
          let outputs = responses.remove(&(self.tx_id.to_string(), self.name.to_string()));
          match serialize(&outputs) {
            Ok(bytes) => Poll::Ready(InvocationResponse::success(tx_id, bytes)),
            Err(e) => Poll::Ready(InvocationResponse::error(tx_id, e.to_string())),
          }
        } else {
          cx.waker().wake_by_ref();
          Poll::Pending
        }
      }
      Err(e) => {
        trace!("Schematic had error");
        Poll::Ready(InvocationResponse::error(tx_id, e.to_string()))
      }
    }
  }
}
