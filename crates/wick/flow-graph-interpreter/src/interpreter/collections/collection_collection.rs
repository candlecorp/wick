use futures::future::BoxFuture;
use serde_json::Value;
use wasmrs_rx::{FluxChannel, Observer};
use wick_interface_types::{CollectionSignature, OperationSignature};
use wick_packet::{CollectionLink, Entity, Invocation, Packet, PacketStream};

use crate::constants::*;
use crate::{BoxError, Collection, HandlerMap};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub(crate) enum Error {
  #[error("Collection with namespace '{0}' not found on this network. This resource handles namespaces: {}", .1.join(", "))]
  CollectionNotFound(String, Vec<String>),
}

#[derive(Debug)]
pub(crate) struct CollectionCollection {
  signature: CollectionSignature,
}

impl CollectionCollection {
  pub(crate) fn new(list: &HandlerMap) -> Self {
    let mut signature = CollectionSignature::new("collections");
    for ns in list.collections().keys() {
      let mut comp_sig = OperationSignature::new(ns.clone());
      comp_sig
        .outputs
        .insert("ref", wick_interface_types::TypeSignature::Link { schemas: vec![] });
      signature.operations.insert(ns.clone(), comp_sig);
    }
    Self { signature }
  }
}

impl Collection for CollectionCollection {
  fn handle(
    &self,
    invocation: Invocation,
    _stream: PacketStream,
    _config: Option<Value>,
  ) -> BoxFuture<Result<PacketStream, BoxError>> {
    trace!(target = %invocation.target, namespace = NS_COLLECTIONS);

    // This handler handles the NS_COLLECTIONS namespace and outputs the entity
    // to link to.
    let target_name = invocation.target.name().to_owned();
    let entity = Entity::collection(invocation.target.name());

    let contains_collection = self.signature.operations.contains_key(target_name);
    let all_collections: Vec<_> = self.signature.operations.inner().keys().cloned().collect();

    Box::pin(async move {
      let port_name = "ref";
      if !contains_collection {
        return Err(Error::CollectionNotFound(entity.name().to_owned(), all_collections).into());
      }
      let flux = FluxChannel::new();

      flux.send(Packet::encode(
        port_name,
        CollectionLink::new(invocation.origin, entity),
      ))?;

      Ok(PacketStream::new(Box::new(flux)))
    })
  }

  fn list(&self) -> &CollectionSignature {
    &self.signature
  }
}
