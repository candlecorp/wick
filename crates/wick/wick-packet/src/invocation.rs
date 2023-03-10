use std::time::SystemTime;

use uuid::Uuid;

use crate::{Entity, InherentData};

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
}

impl Invocation {
  /// Creates an invocation with a new transaction id.
  pub fn new(origin: Entity, target: Entity, inherent: Option<InherentData>) -> Invocation {
    let tx_id = get_uuid();
    let invocation_id = get_uuid();

    Invocation {
      origin,
      target,
      id: invocation_id,
      tx_id,
      inherent,
    }
  }

  /// Creates an invocation with a specific transaction id, to correlate a chain of
  /// invocations.
  pub fn next_tx(&self, origin: Entity, target: Entity) -> Invocation {
    let invocation_id = get_uuid();

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
}

pub(crate) fn get_uuid() -> Uuid {
  Uuid::new_v4()
}

#[cfg(test)]
mod tests {}
