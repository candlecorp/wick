use futures::future::BoxFuture;
use serde_json::Value;
use wasmflow_collection_link::CollectionLink;
use wasmflow_entity::Entity;
use wasmflow_interface::{CollectionSignature, ComponentSignature};
use wasmflow_invocation::Invocation;
use wasmflow_transport::{MessageTransport, TransportStream, TransportWrapper};

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
      let mut comp_sig = ComponentSignature::new(ns.clone());
      comp_sig
        .outputs
        .insert("ref", wasmflow_interface::TypeSignature::Link { schemas: vec![] });
      signature.components.insert(ns.clone(), comp_sig);
    }
    Self { signature }
  }
}

impl Collection for CollectionCollection {
  fn handle(&self, invocation: Invocation, _config: Option<Value>) -> BoxFuture<Result<TransportStream, BoxError>> {
    trace!(target = %invocation.target, id=%invocation.id,namespace = NS_COLLECTIONS);

    // This handler handles the NS_COLLECTIONS namespace and outputs the entity
    // to link to.
    let name = invocation.target.name().to_owned();
    let entity = Entity::collection(&name);

    let contains_collection = self.signature.components.contains_key(&name);
    let all_collections: Vec<_> = self.signature.components.inner().keys().cloned().collect();

    Box::pin(async move {
      let port_name = "ref";
      if !contains_collection {
        return Err(Error::CollectionNotFound(name, all_collections).into());
      }
      let messages = vec![
        TransportWrapper::new(
          port_name,
          MessageTransport::success(&CollectionLink::new(invocation.origin, entity)),
        ),
        TransportWrapper::done(port_name),
      ];

      Ok(TransportStream::new(tokio_stream::iter(messages.into_iter())))
    })
  }

  fn list(&self) -> &CollectionSignature {
    &self.signature
  }
}
