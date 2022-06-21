use std::sync::Arc;

use seeded_random::Seed;
use wasmflow_sdk::v1::transport::TransportStream;
use wasmflow_sdk::v1::Invocation;

use self::error::ExecutionError;
use self::transaction::Transaction;
use super::channel::InterpreterDispatchChannel;
use crate::graph::types::*;
use crate::interpreter::channel::Event;
use crate::{Collection, HandlerMap};

pub(crate) mod error;
mod output_channel;
pub(crate) mod transaction;

type Result<T> = std::result::Result<T, ExecutionError>;

#[derive(Debug, Clone)]
#[must_use]
pub struct SchematicExecutor {
  channel: InterpreterDispatchChannel,
  schematic: Arc<Schematic>,
}

impl SchematicExecutor {
  pub(crate) fn new(schematic: Schematic, channel: InterpreterDispatchChannel) -> Self {
    Self {
      channel,
      schematic: Arc::new(schematic),
    }
  }

  pub(super) fn name(&self) -> &str {
    self.schematic.name()
  }

  #[instrument(skip_all, name = "invocation")]
  pub async fn invoke(
    &self,
    invocation: Invocation,
    seed: Seed,
    collections: Arc<HandlerMap>,
    self_collection: Arc<dyn Collection + Send + Sync>,
  ) -> Result<TransportStream> {
    debug!(schematic = self.name(), %seed,);

    let seed = invocation.seed().map_or(seed, Seed::unsafe_new);

    let mut transaction = Transaction::new(
      self.schematic.clone(),
      invocation,
      self.channel.clone(),
      &collections,
      &self_collection,
      seed,
    );
    trace!(tx_id = %transaction.id(), "invoking schematic");
    let stream = transaction.take_stream().unwrap();
    self.channel.dispatch(Event::tx_start(Box::new(transaction))).await?;
    Ok(stream)
  }
}
