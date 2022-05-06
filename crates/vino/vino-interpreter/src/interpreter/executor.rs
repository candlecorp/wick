use std::sync::Arc;

use vino_random::Seed;
use vino_transport::TransportStream;
use wasmflow_invocation::Invocation;

use self::error::ExecutionError;
use self::transaction::Transaction;
use super::channel::InterpreterDispatchChannel;
use crate::graph::types::*;
use crate::interpreter::channel::Event;
use crate::{HandlerMap, Provider};

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
    providers: Arc<HandlerMap>,
    self_provider: Arc<dyn Provider + Send + Sync>,
  ) -> Result<TransportStream> {
    debug!(schematic = self.name(), %seed,);

    let seed = invocation.seed().map_or(seed, Seed::unsafe_new);

    let mut transaction = Transaction::new(
      self.schematic.clone(),
      invocation,
      self.channel.clone(),
      &providers,
      &self_provider,
      seed,
    );
    trace!(tx_id = %transaction.id(), "invoking schematic");
    let stream = transaction.take_stream().unwrap();
    self.channel.dispatch(Event::tx_start(Box::new(transaction))).await?;
    Ok(stream)
  }
}
