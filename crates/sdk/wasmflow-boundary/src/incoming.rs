/// A map of port name to payload message.
#[derive(Debug)]
pub struct IncomingPayload<P, C, S>
where
  P: std::fmt::Debug,
  C: std::fmt::Debug,
  S: std::fmt::Debug,
{
  id: u32,
  payload: P,
  config: Option<C>,
  state: Option<S>,
}

impl<P, C, S> IncomingPayload<P, C, S>
where
  P: std::fmt::Debug + serde::de::DeserializeOwned,
  C: std::fmt::Debug + serde::de::DeserializeOwned,
  S: std::fmt::Debug + serde::de::DeserializeOwned,
{
  /// Instantiate a new IncomingPayload
  pub fn new(id: u32, payload: P, config: Option<C>, state: Option<S>) -> Self {
    Self {
      id,
      payload,
      config,
      state,
    }
  }

  /// Get the transaction ID associated with this [IncomingPayload].
  #[must_use]
  pub fn id(&self) -> u32 {
    self.id
  }

  /// Get the main payload.
  #[must_use]
  pub fn payload(&self) -> &P {
    &self.payload
  }

  /// Get the configuration associated with the incoming payload.
  #[must_use]
  pub fn config(&self) -> &Option<C> {
    &self.config
  }

  /// Get the state used for the next run of the job.
  #[must_use]
  pub fn state(&self) -> &Option<S> {
    &self.state
  }

  /// Get the state used for the next run of the job.
  #[must_use]
  pub fn into_parts(self) -> (P, Option<C>, Option<S>) {
    (self.payload, self.config, self.state)
  }
}
