use flow_component::{Component, ComponentError, LocalScope};
use wasmrs_rx::Observer;
use wick_interface_types::{ComponentSignature, Field, OperationSignature};
use wick_packet::{ComponentReference, Entity, Invocation, Packet, PacketStream, RuntimeConfig};

use crate::{BoxFuture, HandlerMap};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub(crate) enum Error {
  #[error("Component with id '{0}' not found on this component. This resource handles ids: {}", .1.join(", "))]
  ComponentNotFound(String, Vec<String>),
}

#[derive(Debug)]
pub(crate) struct ComponentComponent {
  signature: ComponentSignature,
}

impl ComponentComponent {
  pub(crate) const ID: &str = "__component__";

  pub(crate) fn new(list: &HandlerMap) -> Self {
    let mut signature = ComponentSignature::new_named("components");
    for ns in list.inner().keys() {
      trace!(id = ns, "interpreter:registering component on 'component' ns");
      let mut comp_sig = OperationSignature::new_named(ns.clone());
      #[allow(deprecated)]
      comp_sig
        .outputs
        .push(Field::new("ref", wick_interface_types::Type::Link { schemas: vec![] }));
      signature.operations.push(comp_sig);
    }
    Self { signature }
  }
}

impl Component for ComponentComponent {
  fn handle(
    &self,
    invocation: Invocation,
    _config: Option<RuntimeConfig>,
    _callback: LocalScope,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    invocation.trace(|| debug!(target = %invocation.target(), namespace = Self::ID));

    // This handler handles the components:: namespace and outputs the entity
    // to link to.
    let target_name = invocation.target().operation_id().to_owned();
    let entity = Entity::component(invocation.target().operation_id());

    let contains_components = self.signature.operations.iter().any(|op| op.name == target_name);
    let all_components: Vec<_> = self.signature.operations.iter().map(|op| op.name.clone()).collect();

    Box::pin(async move {
      let port_name = "ref";
      if !contains_components {
        return Err(ComponentError::new(Error::ComponentNotFound(
          entity.operation_id().to_owned(),
          all_components,
        )));
      }
      let (tx, rx) = invocation.make_response();

      tx.send(Packet::encode(
        port_name,
        ComponentReference::new(invocation.origin().clone(), Entity::component(entity.component_id())),
      ))
      .map_err(ComponentError::new)?;

      Ok(rx)
    })
  }

  fn signature(&self) -> &ComponentSignature {
    &self.signature
  }
}
