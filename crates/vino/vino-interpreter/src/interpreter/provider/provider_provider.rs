use futures::future::BoxFuture;
use serde_json::Value;
use vino_entity::Entity;
use vino_provider::ProviderLink;
use vino_transport::{Invocation, MessageTransport, TransportStream, TransportWrapper};
use vino_types::{ComponentSignature, MapWrapper, ProviderSignature};

use crate::constants::*;
use crate::{BoxError, HandlerMap, Provider};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub(crate) enum Error {
  #[error("Provider with namespace '{0}' not found on this network. This resource handles namespaces: {}", .1.join(", "))]
  ProviderNotFound(String, Vec<String>),
}

#[derive(Debug)]
pub(crate) struct ProviderProvider {
  signature: ProviderSignature,
}

impl ProviderProvider {
  pub(crate) fn new(list: &HandlerMap) -> Self {
    let mut signature = ProviderSignature::new("providers");
    for ns in list.providers().keys() {
      let mut comp_sig = ComponentSignature::new(ns.clone());
      comp_sig
        .outputs
        .insert("ref", vino_types::TypeSignature::Link { schemas: vec![] });
      signature.components.insert(ns.clone(), comp_sig);
    }
    Self { signature }
  }
}

impl Provider for ProviderProvider {
  fn handle(&self, invocation: Invocation, _config: Option<Value>) -> BoxFuture<Result<TransportStream, BoxError>> {
    trace!(target = %invocation.target, id=%invocation.id,namespace = NS_PROVIDERS);

    // This handler handles the NS_PROVIDER namespace and outputs the entity
    // to link to.
    let name = invocation.target.name().to_owned();
    let entity = Entity::provider(&name);

    let contains_provider = self.signature.components.contains_key(&name);
    let all_providers: Vec<_> = self.signature.components.inner().keys().cloned().collect();

    Box::pin(async move {
      let port_name = "ref";
      if !contains_provider {
        return Err(Error::ProviderNotFound(name, all_providers).into());
      }
      let messages = vec![
        TransportWrapper::new(
          port_name,
          MessageTransport::success(&ProviderLink::new(invocation.origin, entity)),
        ),
        TransportWrapper::done(port_name),
      ];

      Ok(TransportStream::new(tokio_stream::iter(messages.into_iter())))
    })
  }

  fn list(&self) -> &ProviderSignature {
    &self.signature
  }
}
