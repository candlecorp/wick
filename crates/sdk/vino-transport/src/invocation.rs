use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vino_entity::Entity;

use crate::TransportMap;

/// A complete invocation request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[must_use]
pub struct Invocation {
  /// The entity that originated the request.
  pub origin: Entity,
  /// The target of the invocation.
  pub target: Entity,
  /// The payload.
  pub payload: TransportMap,
  /// The invocation id.
  pub id: String,
  /// The transaction id, to map together a string of invocations.
  pub tx_id: String,
}

impl Invocation {
  /// Creates an invocation with a new transaction id.
  pub fn new(origin: Entity, target: Entity, payload: TransportMap) -> Invocation {
    let tx_id = get_uuid();
    let invocation_id = get_uuid();

    Invocation {
      origin,
      target,
      payload,
      id: invocation_id,
      tx_id,
    }
  }

  /// Creates an invocation with a specific transaction id, to correlate a chain of.
  /// invocations.
  pub fn next(tx_id: &str, origin: Entity, target: Entity, payload: TransportMap) -> Invocation {
    let invocation_id = get_uuid();

    Invocation {
      origin,
      target,
      payload,
      id: invocation_id,
      tx_id: tx_id.to_owned(),
    }
  }

  /// Creates an invocation with a Test origin.
  pub fn new_test(msg: &str, target: Entity, payload: TransportMap) -> Invocation {
    let tx_id = get_uuid();
    let invocation_id = get_uuid();

    Invocation {
      origin: Entity::test(msg),
      target,
      payload,
      id: invocation_id,
      tx_id,
    }
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

impl TryFrom<(String, String, TransportMap)> for Invocation {
  type Error = crate::Error;

  fn try_from(v: (String, String, TransportMap)) -> Result<Self, Self::Error> {
    Ok(Self::new(Entity::from_str(&v.0)?, Entity::from_str(&v.1)?, v.2))
  }
}

impl TryFrom<(&str, &str, TransportMap)> for Invocation {
  type Error = crate::Error;

  fn try_from(v: (&str, &str, TransportMap)) -> Result<Self, Self::Error> {
    Ok(Self::new(Entity::from_str(v.0)?, Entity::from_str(v.1)?, v.2))
  }
}

pub(crate) fn get_uuid() -> String {
  format!("{}", Uuid::new_v4())
}

#[cfg(test)]
mod tests {}
