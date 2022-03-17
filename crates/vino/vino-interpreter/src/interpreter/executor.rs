use std::sync::Arc;

use uuid::Uuid;
use vino_transport::{Invocation, TransportStream};

use self::error::ExecutionError;
use self::transaction::Transaction;
use super::channel::InterpreterDispatchChannel;
use crate::graph::types::*;
use crate::interpreter::channel::Event;
use crate::{Provider, Providers};

pub(crate) mod error;
mod output_channel;
pub(crate) mod transaction;

type Result<T> = std::result::Result<T, ExecutionError>;

#[derive(Debug, Clone)]
#[must_use]
pub struct SchematicExecutor {
  channel: InterpreterDispatchChannel,
  schematic: Arc<Schematic>,
  tx_id: Uuid,
}

impl SchematicExecutor {
  pub(crate) fn new(schematic: Schematic, channel: InterpreterDispatchChannel) -> Self {
    let tx_id = uuid::Uuid::new_v4();

    Self {
      channel,
      schematic: Arc::new(schematic),
      tx_id,
    }
  }

  pub(super) fn name(&self) -> &str {
    self.schematic.name()
  }

  #[instrument(skip_all, name = "subroutine")]
  pub async fn invoke(
    &self,
    invocation: Invocation,
    providers: Arc<Providers>,
    self_provider: Arc<dyn Provider + Send + Sync>,
  ) -> Result<TransportStream> {
    trace!("running subroutine '{}'", self.name());
    let mut transaction = Transaction::new(
      self.tx_id,
      self.schematic.clone(),
      invocation,
      self.channel.clone(),
      &providers,
      &self_provider,
    );
    let stream = transaction.take_stream().unwrap();
    self.channel.dispatch(Event::tx_start(Box::new(transaction))).await?;
    Ok(stream)
  }
}
