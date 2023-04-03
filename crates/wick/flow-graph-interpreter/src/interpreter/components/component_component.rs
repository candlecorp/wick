use serde_json::Value;
use wasmrs_rx::{FluxChannel, Observer};
use wick_interface_types::{ComponentSignature, Field, OperationSignature};
use wick_packet::{CollectionLink, Entity, Invocation, Packet, PacketStream};

use crate::constants::*;
use crate::{BoxError, BoxFuture, Component, HandlerMap};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub(crate) enum Error {
  #[error("Collection with namespace '{0}' not found on this network. This resource handles namespaces: {}", .1.join(", "))]
  CollectionNotFound(String, Vec<String>),
}

#[derive(Debug)]
pub(crate) struct ComponentComponent {
  signature: ComponentSignature,
}

impl ComponentComponent {
  pub(crate) fn new(list: &HandlerMap) -> Self {
    let mut signature = ComponentSignature::new("collections");
    for ns in list.inner().keys() {
      let mut comp_sig = OperationSignature::new(ns.clone());
      comp_sig.outputs.push(Field::new(
        "ref",
        wick_interface_types::TypeSignature::Link { schemas: vec![] },
      ));
      signature.operations.push(comp_sig);
    }
    Self { signature }
  }
}

impl Component for ComponentComponent {
  fn handle(
    &self,
    invocation: Invocation,
    _stream: PacketStream,
    _config: Option<Value>,
  ) -> BoxFuture<Result<PacketStream, BoxError>> {
    trace!(target = %invocation.target, namespace = NS_COMPONENTS);

    // This handler handles the NS_COLLECTIONS namespace and outputs the entity
    // to link to.
    let target_name = invocation.target.name().to_owned();
    let entity = Entity::component(invocation.target.name());

    let contains_collection = self.signature.operations.iter().any(|op| op.name == target_name);
    let all_collections: Vec<_> = self.signature.operations.iter().map(|op| op.name.clone()).collect();

    Box::pin(async move {
      let port_name = "ref";
      if !contains_collection {
        return Err(Error::CollectionNotFound(entity.name().to_owned(), all_collections).into());
      }
      let flux = FluxChannel::new();

      flux.send(Packet::encode(
        port_name,
        CollectionLink::new(invocation.origin.url(), entity.namespace()),
      ))?;

      Ok(PacketStream::new(Box::new(flux)))
    })
  }

  fn list(&self) -> &ComponentSignature {
    &self.signature
  }
}
