use flow_component::{Component, ComponentError, RuntimeCallback};
use wasmrs_rx::{FluxChannel, Observer};
use wick_interface_types::{ComponentSignature, Field, OperationSignature};
use wick_packet::{ComponentReference, Entity, Invocation, Packet, PacketStream};

use crate::constants::*;
use crate::{BoxFuture, HandlerMap};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub(crate) enum Error {
  #[error("Component with id '{0}' not found on this network. This resource handles ids: {}", .1.join(", "))]
  ComponentNotFound(String, Vec<String>),
}

#[derive(Debug)]
pub(crate) struct ComponentComponent {
  signature: ComponentSignature,
}

impl ComponentComponent {
  pub(crate) fn new(list: &HandlerMap) -> Self {
    let mut signature = ComponentSignature::new("components");
    for ns in list.inner().keys() {
      trace!(id = ns, "interpreter:registering component on 'component' ns");
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
    _config: Option<wick_packet::OperationConfig>,
    _callback: std::sync::Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    trace!(target = %invocation.target, namespace = NS_COMPONENTS);

    // This handler handles the NS_COLLECTIONS namespace and outputs the entity
    // to link to.
    let target_name = invocation.target.operation_id().to_owned();
    let entity = Entity::component(invocation.target.operation_id());

    let contains_collection = self.signature.operations.iter().any(|op| op.name == target_name);
    let all_collections: Vec<_> = self.signature.operations.iter().map(|op| op.name.clone()).collect();

    Box::pin(async move {
      let port_name = "ref";
      if !contains_collection {
        return Err(ComponentError::new(Error::ComponentNotFound(
          entity.operation_id().to_owned(),
          all_collections,
        )));
      }
      let flux = FluxChannel::new();

      flux
        .send(Packet::encode(
          port_name,
          ComponentReference::new(invocation.origin.clone(), Entity::component(entity.component_id())),
        ))
        .map_err(ComponentError::new)?;

      Ok(PacketStream::new(Box::new(flux)))
    })
  }

  fn list(&self) -> &ComponentSignature {
    &self.signature
  }
}
