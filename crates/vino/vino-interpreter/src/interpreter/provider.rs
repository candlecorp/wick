use std::collections::HashMap;
use std::fmt::Debug;

pub(super) mod internal;

use futures::future::BoxFuture;
use vino_transport::{TransportMap, TransportStream};
use vino_types::ComponentMap;

use self::internal::{InternalProvider, INTERNAL_PROVIDER};
use crate::BoxError;

#[derive()]
#[must_use]
pub struct Providers {
  providers: HashMap<String, ProviderNamespace>,
}

impl Default for Providers {
  fn default() -> Self {
    Self::new(Vec::new())
  }
}

impl Debug for Providers {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Providers")
      .field("providers", &self.component_hashmap())
      .finish()
  }
}

impl Providers {
  pub fn new(providers: Vec<ProviderNamespace>) -> Self {
    trace_span!("providers", provider_len = providers.len());
    let mut providers = Self {
      providers: providers.into_iter().map(|p| (p.namespace.clone(), p)).collect(),
    };
    providers.add(ProviderNamespace {
      namespace: INTERNAL_PROVIDER.to_owned(),
      provider: Box::new(InternalProvider::default()),
    });
    providers
  }

  #[must_use]
  pub fn providers(&self) -> &HashMap<String, ProviderNamespace> {
    &self.providers
  }

  #[must_use]
  pub fn component_hashmap(&self) -> HashMap<String, ComponentMap> {
    self
      .providers
      .iter()
      .map(|(name, p)| (name.clone(), p.provider.list().clone()))
      .collect()
  }

  #[must_use]
  #[instrument(name="provider", skip_all, fields(namespace = namespace))]
  pub fn get(&self, namespace: &str) -> Option<&ProviderNamespace> {
    trace!("retrieving provider");
    self.providers.get(namespace)
  }

  #[instrument(name="provider", skip_all, fields(namespace = provider.namespace.as_str()))]
  pub fn add(&mut self, provider: ProviderNamespace) {
    trace!("adding provider");
    self.providers.insert(provider.namespace.clone(), provider);
  }
}

#[derive()]
#[must_use]
pub struct ProviderNamespace {
  pub(crate) namespace: String,
  pub(crate) provider: Box<dyn Provider + Send + Sync>,
}

impl ProviderNamespace {
  pub fn new<T: AsRef<str>>(namespace: T, provider: Box<dyn Provider + Send + Sync>) -> Self {
    Self {
      namespace: namespace.as_ref().to_owned(),
      provider,
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
  fn handle(&self, operation: &str, payload: TransportMap) -> BoxFuture<Result<TransportStream, BoxError>>;
  fn list(&self) -> &ComponentMap;
}
