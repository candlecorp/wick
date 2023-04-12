use std::sync::Arc;

use flow_component::{Component, RuntimeCallback};
use seeded_random::Seed;
use wick_packet::{Invocation, PacketStream};

use self::error::ExecutionError;
use self::transaction::Transaction;
use super::channel::InterpreterDispatchChannel;
use crate::graph::types::*;
use crate::HandlerMap;

pub(crate) mod error;
// mod output_channel;
pub(crate) mod transaction;

type Result<T> = std::result::Result<T, ExecutionError>;

#[derive(Debug, Clone)]
#[must_use]
pub(crate) struct SchematicExecutor {
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

  pub(crate) async fn invoke(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    seed: Seed,
    collections: Arc<HandlerMap>,
    self_collection: Arc<dyn Component + Send + Sync>,
    callback: Arc<RuntimeCallback>,
  ) -> Result<PacketStream> {
    debug!(schematic = self.name(), ?invocation,);

    let seed = invocation.seed().map_or(seed, Seed::unsafe_new);

    let mut transaction = Transaction::new(
      self.schematic.clone(),
      invocation,
      stream,
      self.channel.clone(),
      &collections,
      &self_collection,
      callback,
      seed,
    );
    trace!(tx_id = %transaction.id(), "invoking schematic");
    let stream = transaction.take_stream().unwrap();
    self.channel.dispatch_start(Box::new(transaction)).await;
    Ok(stream)
  }
}
