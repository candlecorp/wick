use std::sync::Arc;

use uuid::Uuid;
use vino_schematic_graph::Schematic;
use vino_transport::{TransportMap, TransportStream};

use self::error::ExecutionError;
use self::transaction::Transaction;
use super::channel::InterpreterDispatchChannel;
use crate::interpreter::channel::InterpreterEvent;

mod buffer;
mod component;
pub(crate) mod error;
mod port;
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
  pub async fn start(&self, inputs: Option<TransportMap>) -> Result<TransportStream> {
    trace!("running subroutine '{}'", self.name());
    let payload = inputs.unwrap_or_default();
    let mut transaction = Transaction::new(self.tx_id, self.schematic.clone(), payload, self.channel.clone());
    let stream = transaction.get_stream().unwrap();
    self
      .channel
      .dispatch(InterpreterEvent::TransactionStart(Box::new(transaction)))
      .await?;
    Ok(stream)
  }
}
