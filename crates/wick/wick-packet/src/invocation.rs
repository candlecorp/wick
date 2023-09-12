use parking_lot::Mutex;
use tracing::{info_span, Span};
use uuid::Uuid;

use crate::{Entity, InherentData, PacketSender, PacketStream};

/// A complete invocation request.
#[derive(Debug)]
#[must_use]
#[allow(clippy::exhaustive_structs)]
pub struct InvocationData {
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
}

impl From<Invocation> for InvocationData {
  fn from(value: Invocation) -> Self {
    value.data
  }
}

impl InvocationData {
  /// Creates an invocation with existing data.
  #[doc(hidden)]
  pub fn new_raw(origin: Entity, target: Entity, id: Uuid, tx_id: Uuid, inherent: InherentData, span: Span) -> Self {
    Self {
      origin,
      target,
      id,
      tx_id,
      inherent,
      span,
    }
  }

  /// Creates an invocation with the passed transaction id.
  pub fn new_with_id(
    tx_id: Uuid,
    origin: impl Into<Entity>,
    target: impl Into<Entity>,
    inherent: InherentData,
    parent: &Span,
  ) -> InvocationData {
    let invocation_id = get_uuid();
    let target = target.into();
    let span =
      info_span!(parent:parent,"invocation",otel.name=format!("invocation:{}", target),id=%invocation_id,tx_id=%tx_id);

    Self {
      origin: origin.into(),
      target,
      id: invocation_id,
      tx_id,
      inherent,
      span,
    }
  }

  pub fn with_stream(self, packets: impl Into<PacketStream>) -> Invocation {
    Invocation {
      data: self,
      packets: Mutex::new(packets.into()),
    }
  }

  /// Make response channels associated with this the invocation.
  pub fn make_response(&self) -> (PacketSender, PacketStream) {
    let (tx, mut rx) = PacketStream::new_channels();
    let span = info_span!(parent:&self.span,"invocation:response", otel.name=format!("invocation:response:{}", self.target), id=%self.id, target=%self.target);

    rx.set_span(span);
    (tx, rx)
  }

  /// Do work within this invocation's trace span.
  pub fn trace<F: FnOnce() -> T, T>(&self, f: F) -> T {
    self.span.in_scope(f)
  }

  /// Get the origin [Entity].
  pub const fn origin(&self) -> &Entity {
    &self.origin
  }

  /// Get the target [Entity].
  pub const fn target(&self) -> &Entity {
    &self.target
  }

  /// Returns the seed for the invocation.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.inherent.seed
  }

  /// Returns the timestamp for the invocation.
  #[must_use]
  pub const fn timestamp(&self) -> u64 {
    self.inherent.timestamp
  }

  pub const fn inherent(&self) -> &InherentData {
    &self.inherent
  }

  /// Return the span associated with the [Invocation].
  #[must_use]
  pub const fn span(&self) -> &Span {
    &self.span
  }

  /// Creates an invocation with a new transaction id.
  #[cfg(feature = "test")]
  pub fn test<T, TE>(name: &str, target: T, inherent: Option<InherentData>) -> Result<InvocationData, crate::ParseError>
  where
    T: TryInto<Entity, Error = TE>,
    TE: std::error::Error + Send + Sync + 'static,
  {
    let inherent = inherent.unwrap_or_else(InherentData::unsafe_default);
    let tx_id = get_uuid();
    let id = get_uuid();

    Ok(Self {
      origin: Entity::test(name),
      target: target
        .try_into()
        .map_err(|e| crate::ParseError::Conversion(Box::new(e)))?,
      inherent,
      span: Span::current(),
      id,
      tx_id,
    })
  }
}

/// A complete invocation request.
#[derive(Debug)]
#[must_use]
pub struct Invocation {
  /// Invocation metadata
  data: InvocationData,
  /// The stream of incoming [crate::Packet]s associated with the invocation.
  packets: Mutex<PacketStream>,
}

impl AsRef<InvocationData> for Invocation {
  fn as_ref(&self) -> &InvocationData {
    &self.data
  }
}

impl Invocation {
  /// Creates a new transaction id.
  #[must_use]
  pub fn new_tx_id() -> Uuid {
    get_uuid()
  }

  /// Creates an invocation with a new transaction id.
  pub fn new(
    origin: impl Into<Entity>,
    target: impl Into<Entity>,
    packets: impl Into<PacketStream>,
    inherent: InherentData,
    parent: &Span,
  ) -> Invocation {
    let tx_id = get_uuid();

    Self::new_with_id(tx_id, origin, target, packets, inherent, parent)
  }

  /// Creates an invocation with the passed transaction id.
  pub fn new_with_id(
    tx_id: Uuid,
    origin: impl Into<Entity>,
    target: impl Into<Entity>,
    packets: impl Into<PacketStream>,
    inherent: InherentData,
    parent: &Span,
  ) -> Invocation {
    let invocation_id = get_uuid();
    let target = target.into();
    let span =
      info_span!(parent:parent,"invocation",otel.name=format!("invocation:{}", target),id=%invocation_id,tx_id=%tx_id);

    let mut packets: PacketStream = packets.into();
    packets.set_span(span.clone());

    Invocation {
      data: InvocationData {
        origin: origin.into(),
        target,
        id: invocation_id,
        tx_id,
        inherent,
        span,
      },
      packets: Mutex::new(packets),
    }
  }

  /// Creates an invocation with a new transaction id.
  #[cfg(feature = "test")]
  pub fn test<T, TE>(
    name: &str,
    target: T,
    packets: impl Into<PacketStream>,
    inherent: Option<InherentData>,
  ) -> Result<Invocation, crate::ParseError>
  where
    T: TryInto<Entity, Error = TE>,
    TE: std::error::Error + Send + Sync + 'static,
  {
    let inherent = inherent.unwrap_or_else(InherentData::unsafe_default);

    Ok(Invocation::new(
      Entity::test(name),
      target
        .try_into()
        .map_err(|e| crate::ParseError::Conversion(Box::new(e)))?,
      packets,
      inherent,
      &Span::current(),
    ))
  }

  #[allow(clippy::missing_const_for_fn)]
  /// Redirect an invocation by changing the target.
  pub fn redirect(self, target: Entity) -> Self {
    Self {
      data: InvocationData { target, ..self.data },
      packets: self.packets,
    }
  }

  #[allow(clippy::missing_const_for_fn)]
  pub fn split(self) -> (InvocationData, PacketStream) {
    (self.data, self.packets.into_inner())
  }

  #[allow(clippy::missing_const_for_fn)]
  pub fn into_stream(self) -> PacketStream {
    self.packets.into_inner()
  }

  pub const fn inherent(&self) -> &InherentData {
    &self.data.inherent
  }

  /// Get the origin [Entity].
  pub const fn origin(&self) -> &Entity {
    &self.data.origin
  }

  /// Get the target [Entity].
  pub const fn target(&self) -> &Entity {
    &self.data.target
  }

  /// Get the transaction id.
  #[must_use]
  pub const fn tx_id(&self) -> Uuid {
    self.data.tx_id
  }

  /// Get the invocation id.
  #[must_use]
  pub const fn id(&self) -> Uuid {
    self.data.id
  }

  /// Get the [Span] associated with the invocation.
  #[must_use]
  pub const fn span(&self) -> &Span {
    &self.data.span
  }

  /// Do work within this invocation's trace span.
  pub fn trace<F: FnOnce() -> T, T>(&self, f: F) -> T {
    self.data.trace(f)
  }

  #[doc(hidden)]
  pub fn set_stream_context(&mut self, context: crate::RuntimeConfig, inherent: InherentData) {
    let mut lock = self.packets.lock();
    lock.set_context(context, inherent);
  }

  pub fn make_response(&self) -> (PacketSender, PacketStream) {
    self.data.make_response()
  }
}

pub(crate) fn get_uuid() -> Uuid {
  Uuid::new_v4()
}

#[cfg(test)]
mod tests {}
