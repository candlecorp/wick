use serde::{Deserialize, Serialize};

use crate::{Entity, PacketStream};

/// An implementation that encapsulates a collection link that components use to call out to components on other Wick collections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct ComponentReference {
  origin: Entity,
  target: Entity,
}

impl ComponentReference {
  /// Constructor for a [ComponentReference]
  pub fn new(origin: Entity, target: Entity) -> Self {
    Self { origin, target }
  }

  #[cfg(feature = "invocation")]
  /// Create an [crate::Invocation] for this component reference.
  pub fn make_invocation(&self, operation: &str, inherent: Option<crate::InherentData>) -> crate::Invocation {
    let target = crate::Entity::operation(self.target.component_id(), operation);

    crate::Invocation::new(self.origin.clone(), target, inherent)
  }

  #[must_use]
  /// Get the URL for the called component
  pub fn get_origin_url(&self) -> String {
    self.origin.url()
  }

  /// Get target component ID.
  #[must_use]
  pub fn get_target_id(&self) -> &str {
    self.target.component_id()
  }

  /// Make a call to the linked collection.
  pub fn call(&self, operation: &str, stream: PacketStream) -> Result<PacketStream, crate::error::Error> {
    // let origin = self.origin.clone();
    // let target = Entity::operation(&self.target_ns, operation).url();

    link_call(self.clone(), operation, stream)
  }
}

impl std::fmt::Display for ComponentReference {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}=>{}", self.origin, self.target)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
struct InvocationPayload {
  reference: ComponentReference,
  operation: String,
}

#[cfg(target_family = "wasm")]
fn link_call(
  compref: ComponentReference,
  target_op: &str,
  input: PacketStream,
) -> Result<PacketStream, crate::error::Error> {
  use futures::StreamExt;
  use wasmrs::RSocket;
  use wasmrs_guest::{FluxChannel, Observer};

  use crate::Packet;
  let mut stream = crate::into_wasmrs(0, input);
  let (tx, rx) = FluxChannel::new_parts();
  let first = InvocationPayload {
    reference: compref,
    operation: target_op.to_owned(),
  };

  let _ = tx.send_result(Packet::encode("", first).into());
  wasmrs_runtime::spawn(async move {
    while let Some(payload) = stream.next().await {
      if let Err(_e) = tx.send_result(payload) {
        // Error sending payload, channel probably closed.
      };
    }
  });

  let response = wasmrs_guest::Host::default().request_channel(rx.boxed());
  Ok(crate::from_raw_wasmrs(response))
}

#[cfg(not(target_family = "wasm"))]
#[allow(clippy::needless_pass_by_value)]
fn link_call(
  _compref: ComponentReference,
  _target_op: &str,
  _input: PacketStream,
) -> Result<PacketStream, crate::error::Error> {
  unimplemented!("Link calls from native components is not implemented yet")
}
