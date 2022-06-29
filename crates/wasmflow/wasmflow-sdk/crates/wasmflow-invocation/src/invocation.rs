use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasmflow_entity::Entity;
use wasmflow_packet::PacketMap;
use wasmflow_transport::TransportMap;

use crate::error::Error;

/// A complete invocation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
pub struct Invocation {
  /// The entity that originated the request.
  pub origin: Entity,
  /// The target of the invocation.
  pub target: Entity,
  /// The payload.
  pub payload: TransportMap,
  /// The invocation id.
  pub id: Uuid,
  /// The transaction id, to map together a string of invocations.
  pub tx_id: Uuid,
  /// Inherent data associated with the transaction.
  pub inherent: Option<InherentData>,
  /// Configuration associated with the invocation.
  pub config: Option<wasmflow_transport::Serialized>,
}

impl Invocation {
  /// Creates an invocation with a new transaction id.
  pub fn new(origin: Entity, target: Entity, payload: TransportMap, inherent: Option<InherentData>) -> Invocation {
    let tx_id = get_uuid();
    let invocation_id = get_uuid();

    Invocation {
      origin,
      target,
      payload,
      id: invocation_id,
      tx_id,
      inherent,
      config: None,
    }
  }

  /// Creates an invocation with a new transaction id.
  pub fn into_v1_parts<C>(self) -> Result<(wasmflow_packet::v1::PacketMap, Option<C>), Error>
  where
    C: std::fmt::Debug + DeserializeOwned,
  {
    let config = match self.config {
      Some(v) => Some(
        v.deserialize()
          .map_err(|e| Error::IncomingPayload(format!("could not deserialize config: {}", e)))?,
      ),
      None => None,
    };

    Ok((self.payload.into_v1_map(), config))
  }

  /// Creates an invocation with a specific transaction id, to correlate a chain of.
  /// invocations.
  pub fn next(
    tx_id: Uuid,
    origin: Entity,
    target: Entity,
    payload: TransportMap,
    inherent: Option<InherentData>,
  ) -> Invocation {
    let invocation_id = get_uuid();

    Invocation {
      origin,
      target,
      payload,
      id: invocation_id,
      tx_id,
      inherent,
      config: None,
    }
  }

  /// Creates an invocation with a Test origin.
  pub fn new_test(
    msg: &str,
    target: Entity,
    payload: impl Into<PacketMap>,
    inherent: Option<InherentData>,
  ) -> Invocation {
    let payload = payload.into();
    let tx_id = get_uuid();
    let invocation_id = get_uuid();

    Invocation {
      origin: Entity::test(msg),
      target,
      payload: payload.into(),
      id: invocation_id,
      tx_id,
      inherent,
      config: None,
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
/// Data inherent to an invocation. Meant to be supplied by a runtime, not a user.
#[must_use]
pub struct InherentData {
  /// The seed to associate with an invocation.
  pub seed: u64,
  /// The timestamp to associate with an invocation.
  pub timestamp: u64,
}

impl InherentData {
  /// Constructor for [InherentData]
  pub fn new(seed: u64, timestamp: u64) -> Self {
    Self { seed, timestamp }
  }
}

pub(crate) fn get_uuid() -> Uuid {
  Uuid::new_v4()
}

#[cfg(test)]
mod tests {}
