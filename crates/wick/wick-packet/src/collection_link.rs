use std::pin::Pin;

use futures::{Future, StreamExt};
use serde::{Deserialize, Serialize};
use wasmrs::{Metadata, PayloadError, RSocket};
use wasmrs_frames::RawPayload;
use wasmrs_guest::{FluxChannel, Observer};

use crate::{from_wasmrs, Entity, PacketStream};

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

  #[must_use]
  /// Get the URL for the called component
  pub fn get_origin_url(&self) -> &str {
    &self.origin
  }

  /// Make a call to the linked collection.
  pub fn call(
    &self,
    component: &str,
    stream: PacketStream,
  ) -> Pin<Box<impl Future<Output = Result<PacketStream, crate::error::Error>>>> {
    let origin = self.origin.clone();
    let target = Entity::operation(&self.target_ns, component).url();
    Box::pin(async move {
      let stream = link_call(&origin, &target, stream).await?;

      Ok(stream)
    })
  }
}

impl std::fmt::Display for CollectionLink {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}=>{}", self.origin, self.target_ns)
  }
}

// #[cfg(target_arch = "wasm32")]
async fn link_call(origin: &str, target: &str, mut input: PacketStream) -> Result<PacketStream, crate::error::Error> {
  let (host_tx, host_rx) = FluxChannel::<RawPayload, PayloadError>::new_parts();
  let host_stream = wasmrs_guest::Host::default().request_channel(Box::new(host_rx));
  let md = Metadata::new(0);
  let invocation = wasmrs_codec::messagepack::serialize(&(origin, target)).unwrap();
  host_tx
    .send(RawPayload::new_data(Some(md.encode()), Some(invocation.into())))
    .unwrap();

  wasmrs_runtime::spawn(async move {
    while let Some(Ok(payload)) = input.next().await {
      host_tx.send_result(payload.into()).unwrap();
    }
  });

  Ok(from_wasmrs(host_stream))
}

// #[cfg(not(target_arch = "wasm32"))]
// async fn link_call(_origin: &str, _target: &str, _payload: PacketStream) -> Result<PacketStream, crate::error::Error> {
//   unimplemented!("Link calls from native collections is not implemented yet")
// }
