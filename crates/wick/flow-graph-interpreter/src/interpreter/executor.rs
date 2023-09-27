use std::sync::Arc;

use flow_component::LocalScope;
use seeded_random::Seed;
use wick_packet::{Invocation, PacketStream, RuntimeConfig};

use self::context::ExecutionContext;
use self::error::ExecutionError;
use super::channel::InterpreterDispatchChannel;
use super::components::self_component::SelfComponent;
use crate::graph::types::*;
use crate::HandlerMap;

pub(crate) mod error;
// mod output_channel;
pub(crate) mod context;

type Result<T> = std::result::Result<T, ExecutionError>;

#[derive(Debug, Clone)]
#[must_use]
pub(crate) struct SchematicExecutor {
  channel: InterpreterDispatchChannel,
  root_config: Option<RuntimeConfig>,
  schematic: Arc<Schematic>,
}

impl SchematicExecutor {
  pub(crate) fn new(
    schematic: Schematic,
    channel: InterpreterDispatchChannel,
    root_config: Option<RuntimeConfig>,
  ) -> Self {
    Self {
      channel,
      schematic: Arc::new(schematic),
      root_config,
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
    self_component: SelfComponent,
    config: Option<RuntimeConfig>,
    callback: LocalScope,
  ) -> Result<PacketStream> {
    invocation
      .trace(|| debug!(operation = self.name(), origin=%invocation.origin(),target=%invocation.target(),"invoking"));

    let (invocation, stream) = invocation.split();

    let seed = Seed::unsafe_new(invocation.seed());

    let (ctx, output_stream) = ExecutionContext::new(
      self.schematic.clone(),
      &invocation,
      self.channel.clone(),
      &components,
      &self_component,
      callback,
      self.root_config.clone(),
      config,
      seed,
    );
    ExecutionContext::run(ctx, stream);
    Ok(output_stream)
  }
}
