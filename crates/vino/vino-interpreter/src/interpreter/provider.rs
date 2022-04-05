use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

pub(super) mod core_provider;
pub(crate) mod internal_provider;
pub(super) mod provider_provider;
pub(super) mod schematic_provider;

use futures::future::BoxFuture;
use serde_json::Value;
use vino_transport::{Invocation, TransportMap, TransportStream};
use vino_types::{ProviderMap, ProviderSignature};

use crate::constants::*;
use crate::interpreter::provider::core_provider::CoreProvider;
use crate::interpreter::provider::internal_provider::InternalProvider;
use crate::BoxError;

#[derive()]
#[must_use]
pub struct HandlerMap {
  providers: HashMap<String, ProviderNamespace>,
}

impl Default for HandlerMap {
  fn default() -> Self {
    Self::new(Vec::new())
  }
}

impl Debug for HandlerMap {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Providers")
      .field("providers", &self.provider_signatures())
      .finish()
  }
}

impl HandlerMap {
  pub fn new(providers: Vec<ProviderNamespace>) -> Self {
    trace_span!("providers", provider_len = providers.len());
    let mut map = Self {
      providers: Default::default(),
    };
    for provider in providers {
      map.add(provider);
    }

    map.add(ProviderNamespace {
      namespace: NS_INTERNAL.to_owned(),
      provider: Arc::new(Box::new(InternalProvider::default())),
    });

    map.add(ProviderNamespace {
      namespace: NS_CORE.to_owned(),
      provider: Arc::new(Box::new(CoreProvider::default())),
    });
    map
  }

  #[must_use]
  pub fn providers(&self) -> &HashMap<String, ProviderNamespace> {
    &self.providers
  }

  pub fn provider_signatures(&self) -> ProviderMap {
    self
      .providers
      .iter()
      .map(|(name, p)| (name.clone(), p.provider.list().clone()))
      .collect::<HashMap<String, ProviderSignature>>()
      .into()
  }

  #[must_use]
  pub fn get(&self, namespace: &str) -> Option<&ProviderNamespace> {
    trace!(namespace, "retrieving provider");
    self.providers.get(namespace)
  }

  pub fn add(&mut self, provider: ProviderNamespace) {
    trace!(namespace = %provider.namespace, "adding provider");
    self.providers.insert(provider.namespace.clone(), provider);
  }
}

#[derive(Clone)]
#[must_use]
pub struct ProviderNamespace {
  pub(crate) namespace: String,
  pub(crate) provider: Arc<Box<dyn Provider + Send + Sync>>,
}

impl ProviderNamespace {
  pub fn new<T: AsRef<str>>(namespace: T, provider: Box<dyn Provider + Send + Sync>) -> Self {
    Self {
      namespace: namespace.as_ref().to_owned(),
      provider: Arc::new(provider),
    }
  }
}

impl Debug for ProviderNamespace {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ProviderNamespace")
      .field("namespace", &self.namespace)
      .field("provider", &self.provider.list())
      .finish()
  }
}

pub trait Provider {
  fn handle(&self, invocation: Invocation, data: Option<Value>) -> BoxFuture<Result<TransportStream, BoxError>>;
  fn list(&self) -> &ProviderSignature;
  fn shutdown(&self) -> BoxFuture<Result<(), BoxError>> {
    // Override if you need a more explicit shutdown.
    Box::pin(async move { Ok(()) })
  }
}

pub trait Component {
  fn handle(&self, payload: TransportMap, data: Option<Value>) -> BoxFuture<Result<TransportStream, BoxError>>;
}
