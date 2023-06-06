use std::time::SystemTime;

use tracing::{debug_span, Span};
use uuid::Uuid;

use crate::{Entity, InherentData, PacketSender, PacketStream, ParseError};

/// A complete invocation request.
#[derive(Debug)]
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
  pub inherent: InherentData,
  /// The trace span associated with the invocation.
  pub span: Span,
  /// The stream of incoming [crate::Packet]s associated with the invocation.
  pub packets: PacketStream,
}

impl Invocation {
  /// Creates an invocation with a new transaction id.
  pub fn new(
    origin: impl Into<Entity>,
    target: impl Into<Entity>,
    packets: impl Into<PacketStream>,
    inherent: InherentData,
    following_span: &Span,
  ) -> Invocation {
    let tx_id = get_uuid();
    let invocation_id = get_uuid();
    let span = debug_span!("invocation",id=%invocation_id,tx_id=%tx_id);
    span.follows_from(following_span.id());
    let mut packets: PacketStream = packets.into();
    packets.set_span(span.clone());

    Invocation {
      origin: origin.into(),
      target: target.into(),
      id: invocation_id,
      tx_id,
      inherent,
      span,
      packets,
    }
  }

  /// Creates an invocation with a new transaction id.
  pub fn new_empty(
    origin: impl Into<Entity>,
    target: impl Into<Entity>,
    inherent: InherentData,
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
      packets: PacketStream::empty(),
    }
  }

  /// Creates an invocation with a new transaction id.
  pub fn try_new<O, T, OE, TE>(
    origin: O,
    target: T,
    packets: impl Into<PacketStream>,
    inherent: InherentData,
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
      packets,
      inherent,
      following_span,
    ))
  }

  /// Creates an invocation with a new transaction id.
  #[cfg(feature = "test")]
  pub fn test<T, TE>(
    name: &str,
    target: T,
    packets: impl Into<PacketStream>,
    inherent: Option<InherentData>,
  ) -> Result<Invocation, ParseError>
  where
    T: TryInto<Entity, Error = TE>,
    TE: std::error::Error + Send + Sync + 'static,
  {
    let inherent = inherent.unwrap_or_else(InherentData::unsafe_default);

    Ok(Invocation::new(
      Entity::test(name),
      target.try_into().map_err(|e| ParseError::Conversion(Box::new(e)))?,
      packets,
      inherent,
      &Span::current(),
    ))
  }

  /// Creates an invocation with a specific transaction id, to correlate a chain of
  /// invocations.
  pub fn next_tx(&self, origin: Entity, target: Entity) -> Invocation {
    let invocation_id = get_uuid();
    let span = debug_span!("invocation",id=%invocation_id, %target);
    span.follows_from(&self.span);
    let mut packets: PacketStream = PacketStream::empty();
    packets.set_span(span.clone());

    Invocation {
      origin,
      target,
      id: invocation_id,
      tx_id: self.tx_id,
      inherent: InherentData {
        seed: seeded_random::Seed::unsafe_new(self.inherent.seed).rng().gen(),
        timestamp: SystemTime::now()
          .duration_since(SystemTime::UNIX_EPOCH)
          .unwrap()
          .as_millis() as u64,
      },
      packets: PacketStream::empty(),
      span,
    }
  }

  pub fn eject_stream(&mut self) -> PacketStream {
    std::mem::replace(&mut self.packets, PacketStream::empty())
  }

  pub fn attach_stream(&mut self, packets: impl Into<PacketStream>) {
    let mut stream: PacketStream = packets.into();
    stream.set_span(self.span.clone());
    let _ = std::mem::replace(&mut self.packets, stream);
  }

  /// Get the seed associated with an invocation if it exists.
  #[must_use]
  pub fn seed(&self) -> u64 {
    self.inherent.seed
  }

  /// Get the timestamp associated with an invocation if it exists.
  #[must_use]
  pub fn timestamp(&self) -> u64 {
    self.inherent.timestamp
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

  pub fn make_response(&self) -> (PacketSender, PacketStream) {
    let (tx, mut rx) = PacketStream::new_channels();
    let span = debug_span!("invocation-response", id=%self.id, target=%self.target);
    span.follows_from(&self.span);
    rx.set_span(span);
    (tx, rx)
  }
}

pub(crate) fn get_uuid() -> Uuid {
  Uuid::new_v4()
}

#[cfg(test)]
mod tests {}
