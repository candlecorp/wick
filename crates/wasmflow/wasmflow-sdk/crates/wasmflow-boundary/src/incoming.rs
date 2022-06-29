/// A map of port name to payload message.
#[derive(Debug)]
pub struct IncomingPayload<P, C>
where
  P: std::fmt::Debug,
  C: std::fmt::Debug,
{
  id: u32,
  payload: P,
  config: Option<C>,
}

impl<P, C> IncomingPayload<P, C>
where
  P: std::fmt::Debug + serde::de::DeserializeOwned,
  C: std::fmt::Debug + serde::de::DeserializeOwned,
{
  /// Instantiate a new IncomingPayload
  pub fn new(id: u32, payload: P, config: Option<C>) -> Self {
    Self { id, payload, config }
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
  pub fn into_parts(self) -> (P, Option<C>) {
    (self.payload, self.config)
  }
}
