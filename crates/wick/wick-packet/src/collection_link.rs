use serde::{Deserialize, Serialize};

use crate::{Entity, Result};

/// An implementation that encapsulates a collection link that components use to call out to components on other Wick collections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct ComponentReference {
  origin: Entity,
  target: Entity,
}

impl ComponentReference {
  /// Constructor for a [ComponentReference]
  pub const fn new(origin: Entity, target: Entity) -> Self {
    Self { origin, target }
  }

  #[cfg(feature = "invocation")]
  /// Create an [crate::Invocation] for this component reference.
  pub fn to_invocation(
    &self,
    operation: &str,
    packets: impl Into<crate::PacketStream>,
    inherent: crate::InherentData,
    parent: &tracing::Span,
  ) -> crate::Invocation {
    let target = crate::Entity::operation(self.target.component_id(), operation);

    crate::Invocation::new(self.origin.clone(), target, packets, inherent, parent)
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
  pub fn call(
    &self,
    operation: &str,
    stream: wasmrs_rx::BoxFlux<wasmrs::RawPayload, wasmrs_frames::PayloadError>,
    config: Option<crate::RuntimeConfig>,
    previous_inherent: crate::InherentData,
  ) -> Result<wasmrs_rx::BoxFlux<wasmrs::Payload, wasmrs_frames::PayloadError>> {
    link_call(self.clone(), operation, stream, config, previous_inherent)
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
  mut stream: wasmrs_rx::BoxFlux<wasmrs::RawPayload, wasmrs_frames::PayloadError>,
  config: Option<crate::RuntimeConfig>,
  previous_inherent: crate::InherentData,
) -> Result<wasmrs_rx::BoxFlux<wasmrs::Payload, wasmrs_frames::PayloadError>> {
  use tokio_stream::StreamExt;
  use wasmrs::RSocket;
  use wasmrs_guest::{FluxChannel, Observer};

  let (tx, rx) = FluxChannel::new_parts();
  let first = crate::ContextTransport {
    config,
    invocation: Some(crate::InvocationRequest {
      reference: compref,
      operation: target_op.to_owned(),
    }),
    inherent: previous_inherent,
  };

  let _ = tx.send_result(crate::Packet::encode("", first).into());
  let _ = wasmrs_guest::runtime::spawn("comp_ref", async move {
    loop {
      if let Some(payload) = stream.next().await {
        if let Err(_e) = tx.send_result(payload) {
          // Error sending payload, channel probably closed.
        };
      } else {
        break;
      }
    }
  });

  Ok(Box::pin(wasmrs_guest::Host::default().request_channel(rx.boxed()).map(
    |r| {
      r.and_then(|r| {
        wasmrs::Payload::try_from(r).map_err(|e| wasmrs_frames::PayloadError::application_error(e.to_string(), None))
      })
    },
  )))
}

#[cfg(not(target_family = "wasm"))]
#[allow(clippy::needless_pass_by_value)]
fn link_call(
  _compref: ComponentReference,
  _target_op: &str,
  _input: wasmrs_rx::BoxFlux<wasmrs::RawPayload, wasmrs_frames::PayloadError>,
  _config: Option<crate::RuntimeConfig>,
  _previous_inherent: crate::InherentData,
) -> Result<wasmrs_rx::BoxFlux<wasmrs::Payload, wasmrs_frames::PayloadError>> {
  unimplemented!("Link calls from native components is not implemented yet")
}
