use std::time::SystemTime;

use tracing::{debug_span, Span};
use uuid::Uuid;

use crate::{Entity, InherentData, ParseError};

/// A complete invocation request.
#[derive(Debug, Clone)]
#[must_use]
pub struct Invocation {
  /// The entity that initiated the request.
  pub origin: Entity,
  /// The target of the invocation.
  pub target: Entity,
  /// The invocation id.
  pub id: Uuid,
  /// The transaction id, to map together a string of invocations.
  pub tx_id: Uuid,
  /// Inherent data associated with the transaction.
  pub inherent: Option<InherentData>,
  /// The trace span associated with the invocation.
  pub span: Span,
}

impl Invocation {
  /// Creates an invocation with a new transaction id.
  pub fn new(
    origin: impl Into<Entity>,
    target: impl Into<Entity>,
    inherent: Option<InherentData>,
    following_span: &Span,
  ) -> Invocation {
    let tx_id = get_uuid();
    let invocation_id = get_uuid();
    let span = debug_span!("invocation",id=%invocation_id,tx_id=%tx_id);
    span.follows_from(following_span.id());

    Invocation {
      origin: origin.into(),
      target: target.into(),
      id: invocation_id,
      tx_id,
      inherent,
      span,
    }
  }

  /// Creates an invocation with a new transaction id.
  pub fn try_new<O, T, OE, TE>(
    origin: O,
    target: T,
    inherent: Option<InherentData>,
    following_span: &Span,
  ) -> Result<Invocation, ParseError>
  where
    O: TryInto<Entity, Error = OE>,
    OE: std::error::Error + Send + Sync + 'static,
    T: TryInto<Entity, Error = TE>,
    TE: std::error::Error + Send + Sync + 'static,
  {
    Ok(Invocation::new(
      origin.try_into().map_err(|e| ParseError::Conversion(Box::new(e)))?,
      target.try_into().map_err(|e| ParseError::Conversion(Box::new(e)))?,
      inherent,
      following_span,
    ))
  }

  /// Creates an invocation with a new transaction id.
  pub fn test<T, TE>(name: &str, target: T, inherent: Option<InherentData>) -> Result<Invocation, ParseError>
  where
    T: TryInto<Entity, Error = TE>,
    TE: std::error::Error + Send + Sync + 'static,
  {
    Ok(Invocation::new(
      Entity::test(name),
      target.try_into().map_err(|e| ParseError::Conversion(Box::new(e)))?,
      inherent,
      &Span::current(),
    ))
  }

  /// Creates an invocation with a specific transaction id, to correlate a chain of
  /// invocations.
  pub fn next_tx(&self, origin: Entity, target: Entity) -> Invocation {
    let invocation_id = get_uuid();
    let span = debug_span!("invocation",id=%invocation_id);
    span.follows_from(&self.span);

    Invocation {
      origin,
      target,
      id: invocation_id,
      tx_id: self.tx_id,
      inherent: self.inherent.map(|i| InherentData {
        seed: seeded_random::Seed::unsafe_new(i.seed).rng().gen(),
        timestamp: SystemTime::now()
          .duration_since(SystemTime::UNIX_EPOCH)
          .unwrap()
          .as_millis() as u64,
      }),
      span,
    }
  }

  /// Get the seed associated with an invocation if it exists.
  #[must_use]
  pub fn seed(&self) -> Option<u64> {
    self.inherent.map(|i| i.seed)
  }

  /// Get the timestamp associated with an invocation if it exists.
  #[must_use]
  pub fn timestamp(&self) -> Option<u64> {
    self.inherent.map(|i| i.timestamp)
  }

  /// Utility function to get the target [Entity] URL.
  #[must_use]
  pub fn target_url(&self) -> String {
    self.target.url()
  }

  /// Utility function to get the origin [Entity] URL.
  #[must_use]
  pub fn origin_url(&self) -> String {
    self.origin.url()
  }

  /// Do work within this invocation's trace span.
  pub fn trace<F: FnOnce() -> T, T>(&self, f: F) -> T {
    self.span.in_scope(f)
  }
}

pub(crate) fn get_uuid() -> Uuid {
  Uuid::new_v4()
}

#[cfg(test)]
mod tests {}
