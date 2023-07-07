use std::sync::Arc;

use flow_component::{Component, RuntimeCallback};
use seeded_random::Seed;
use wick_packet::{Invocation, PacketStream};

use self::error::ExecutionError;
use self::transaction::Transaction;
use super::channel::InterpreterDispatchChannel;
use crate::graph::types::*;
use crate::graph::LiquidOperationConfig;
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

  #[allow(clippy::unused_async)]
  pub(crate) async fn invoke(
    &self,
    invocation: Invocation,
    components: Arc<HandlerMap>,
    self_component: Arc<dyn Component + Send + Sync>,
    config: LiquidOperationConfig,
    callback: Arc<RuntimeCallback>,
  ) -> Result<PacketStream> {
    invocation
      .trace(|| debug!(operation = self.name(), origin=%invocation.origin,target=%invocation.target,"invoking"));

    let seed = Seed::unsafe_new(invocation.seed());

    let mut transaction = Transaction::new(
      self.schematic.clone(),
      invocation,
      self.channel.clone(),
      &components,
      &self_component,
      callback,
      config,
      seed,
    );
    let stream = transaction.take_stream().unwrap();
    self.channel.dispatch_start(Box::new(transaction));
    Ok(stream)
  }
}
