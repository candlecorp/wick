use serde::{
  Deserialize,
  Serialize,
};
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
  pub msg: TransportMap,
  /// The invocation id.
  pub id: String,
  /// The transaction id, to map together a string of invocations.
  pub tx_id: String,
}

impl Invocation {
  /// Creates an invocation with a new transaction id.

  pub fn new(origin: Entity, target: Entity, msg: TransportMap) -> Invocation {
    let tx_id = get_uuid();
    let invocation_id = get_uuid();

    Invocation {
      origin,
      target,
      msg,
      id: invocation_id,
      tx_id,
    }
  }

  /// Creates an invocation with a specific transaction id, to correlate a chain of.
  /// invocations.
  pub fn next(tx_id: &str, origin: Entity, target: Entity, msg: TransportMap) -> Invocation {
    let invocation_id = get_uuid();

    Invocation {
      origin,
      target,
      msg,
      id: invocation_id,
      tx_id: tx_id.to_owned(),
    }
  }
}

pub(crate) fn get_uuid() -> String {
  format!("{}", Uuid::new_v4())
}

#[cfg(test)]
mod tests {}
