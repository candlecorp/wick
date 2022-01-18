use crate::dev::prelude::*;

#[derive(Debug, Clone)]
pub(crate) struct OutputMessage {
  pub(crate) port: ConnectionTargetDefinition,
  pub(crate) tx_id: String,
  pub(crate) payload: MessageTransport,
}

impl OutputMessage {
  pub(crate) fn new<T: AsRef<str>>(
    tx_id: T,
    port: ConnectionTargetDefinition,
    payload: MessageTransport,
  ) -> Self {
    Self {
      tx_id: tx_id.as_ref().to_owned(),
      port,
      payload,
    }
  }
}
