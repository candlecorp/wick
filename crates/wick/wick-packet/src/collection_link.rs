use serde::{Deserialize, Serialize};

use crate::{Entity, PacketStream};

/// An implementation that encapsulates a collection link that components use to call out to components on other Wick collections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct CollectionLink {
  origin: String,
  target_ns: String,
}

impl CollectionLink {
  /// Constructor for a [CollectionLink]
  pub fn new(from: impl AsRef<str>, to: impl AsRef<str>) -> Self {
    Self {
      origin: from.as_ref().to_owned(),
      target_ns: to.as_ref().to_owned(),
    }
  }

  #[cfg(feature = "invocation")]
  /// Create an [crate::Invocation] for this component reference.
  pub fn make_invocation(&self, operation: &str, inherent: Option<crate::InherentData>) -> crate::Invocation {
    use std::str::FromStr;
    crate::Invocation::new(
      Entity::from_str(&self.origin).unwrap(),
      Entity::operation(&self.target_ns, operation),
      inherent,
    )
  }

  #[must_use]
  /// Get the URL for the called component
  pub fn get_origin_url(&self) -> &str {
    &self.origin
  }

  /// Get target component ID.
  #[must_use]
  pub fn get_target_id(&self) -> &str {
    &self.target_ns
  }

  /// Make a call to the linked collection.
  pub fn call(&self, component: &str, stream: PacketStream) -> Result<PacketStream, crate::error::Error> {
    let origin = self.origin.clone();
    let target = Entity::operation(&self.target_ns, component).url();

    link_call(&origin, &target, stream)
  }
}

impl std::fmt::Display for CollectionLink {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}=>{}", self.origin, self.target_ns)
  }
}

#[cfg(target_family = "wasm")]
fn link_call(_origin: &str, _target: &str, input: PacketStream) -> Result<PacketStream, crate::error::Error> {
  // use futures::StreamExt;
  use wasmrs::RSocket;
  // use wasmrs_guest::Observer;

  // let (host_tx, host_rx) = wasmrs_guest::FluxChannel::<wasmrs_frames::RawPayload, wasmrs::PayloadError>::new_parts();
  // let host_stream = wasmrs_guest::Host::default().request_channel(Box::new(host_rx));
  // let md = wasmrs::Metadata::new(0);
  let stream = crate::into_wasmrs(0, input);
  let response = wasmrs_guest::Host::default().request_channel(stream);
  Ok(crate::from_wasmrs(response))

  // let invocation = wasmrs_codec::messagepack::serialize(&(origin, target)).unwrap();
  // host_tx
  //   .send(wasmrs_frames::RawPayload::new_data(
  //     Some(md.encode()),
  //     Some(invocation.into()),
  //   ))
  //   .unwrap();

  // wasmrs_runtime::spawn(async move {
  //   while let Some(Ok(payload)) = input.next().await {
  //     host_tx.send_result(payload.into()).unwrap();
  //   }
  // });

  // Ok(crate::from_wasmrs(host_stream))
}

#[cfg(not(target_family = "wasm"))]
#[allow(clippy::needless_pass_by_value)]
fn link_call(_origin: &str, _target: &str, _payload: PacketStream) -> Result<PacketStream, crate::error::Error> {
  unimplemented!("Link calls from native components is not implemented yet")
}
