use crate::dev::prelude::*;

#[derive(Clone)]
pub(crate) struct ShortCircuit {
  pub(crate) tx_id: String,
  pub(crate) instance: String,
  pub(crate) payload: MessageTransport,
}

impl ShortCircuit {
  pub(crate) fn new<T, U>(tx_id: T, instance: U, payload: MessageTransport) -> Self
  where
    T: AsRef<str>,
    U: AsRef<str>,
  {
    Self {
      tx_id: tx_id.as_ref().to_owned(),
      instance: instance.as_ref().to_owned(),
      payload,
    }
  }
}
