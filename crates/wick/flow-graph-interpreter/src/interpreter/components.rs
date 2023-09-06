use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub(crate) mod component;
pub(crate) mod core;
pub(crate) mod internal;
pub(crate) mod null;
pub(crate) mod self_component;

use flow_component::Component;
use wick_interface_types::{ComponentSignature, OperationSignature};

use self::core::CoreComponent;
use self::internal::InternalComponent;
use crate::error::InterpreterError;
use crate::graph::types::Network;
use crate::SharedHandler;

pub(crate) type ComponentMap = HashMap<String, ComponentSignature>;

#[derive(Debug)]
#[must_use]
pub struct HandlerMap {
  components: HashMap<String, NamespaceHandler>,
}

impl Default for HandlerMap {
  fn default() -> Self {
    Self::new(Vec::new()).unwrap()
  }
}

impl HandlerMap {
  pub fn new(components: Vec<NamespaceHandler>) -> Result<Self, InterpreterError> {
    let mut map = Self {
      components: Default::default(),
    };
    for component in components {
      map.add(component)?;
    }

    map.add(NamespaceHandler::new(
      InternalComponent::ID,
      Box::new(InternalComponent::default()),
    ))?;

    Ok(map)
  }

  pub(crate) fn add_core(&mut self, network: &Network) -> Result<(), InterpreterError> {
    self.add(NamespaceHandler::new(
      CoreComponent::ID,
      Box::new(CoreComponent::new(network, self)?),
    ))
  }

  #[must_use]
  pub const fn inner(&self) -> &HashMap<String, NamespaceHandler> {
    &self.components
  }

  #[must_use]
  pub fn component_signatures(&self) -> ComponentMap {
    self
      .components
      .iter()
      .map(|(name, p)| (name.clone(), p.component.signature().clone()))
      .collect::<HashMap<String, ComponentSignature>>()
  }

  #[must_use]
  pub fn get(&self, namespace: &str) -> Option<&NamespaceHandler> {
    self.components.get(namespace)
  }

  pub fn add(&mut self, component: NamespaceHandler) -> Result<(), InterpreterError> {
    if self.components.contains_key(&component.namespace) {
      return Err(InterpreterError::DuplicateNamespace(component.namespace));
    }
    self.components.insert(component.namespace.clone(), component);
    Ok(())
  }

  pub(crate) fn keys(&self) -> Vec<String> {
    self.components.keys().cloned().collect()
  }

  #[allow(unused)]
  pub(crate) fn get_signature(&self, namespace: &str) -> Option<&ComponentSignature> {
    self.components.get(namespace).map(|c| c.component.signature())
  }

  pub(crate) fn get_op_signature(&self, namespace: &str, name: &str) -> Option<&OperationSignature> {
    self
      .components
      .get(namespace)
      .and_then(|c| c.component.signature().get_operation(name))
  }

  pub(crate) fn get_op_list(&self, namespace: &str) -> Vec<&str> {
    self
      .components
      .get(namespace)
      .map(|c| {
        c.component
          .signature()
          .operations
          .iter()
          .map(|op| op.name.as_str())
          .collect::<Vec<_>>()
      })
      .unwrap_or_default()
  }
}

pub(crate) fn dyn_component_id(name: &str, schematic: &str, instance: &str) -> String {
  format!("{}<{}::{}>", name, schematic, instance)
}

pub(crate) fn reconcile_op_id(ns: &str, name: &str, schematic: &str, instance: &str) -> String {
  if ns == CoreComponent::ID && core::DYNAMIC_OPERATIONS.contains(&name) {
    dyn_component_id(name, schematic, instance)
  } else {
    name.to_owned()
  }
}

#[derive(Clone)]
#[must_use]
pub struct NamespaceHandler {
  pub(crate) namespace: String,
  pub(crate) component: SharedHandler,
  pub(crate) exposed: Arc<AtomicBool>,
}

impl NamespaceHandler {
  pub fn new<T: AsRef<str>>(namespace: T, component: Box<dyn Component + Send + Sync>) -> Self {
    Self {
      namespace: namespace.as_ref().to_owned(),
      component: Arc::new(component),
      exposed: Arc::new(AtomicBool::new(false)),
    }
  }

  pub fn new_from_shared<T: AsRef<str>>(namespace: T, component: Arc<Box<dyn Component + Send + Sync>>) -> Self {
    Self {
      namespace: namespace.as_ref().to_owned(),
      component,
      exposed: Arc::new(AtomicBool::new(false)),
    }
  }

  #[must_use]
  pub fn namespace(&self) -> &str {
    &self.namespace
  }

  #[must_use]
  pub fn component(&self) -> &SharedHandler {
    &self.component
  }

  pub fn expose(&self) {
    self.exposed.store(true, std::sync::atomic::Ordering::Relaxed);
  }

  #[must_use]
  pub fn is_exposed(&self) -> bool {
    self.exposed.load(std::sync::atomic::Ordering::Relaxed)
  }
}

impl Debug for NamespaceHandler {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("NamespaceHandler")
      .field("namespace", &self.namespace)
      .field("component", &self.component.signature())
      .finish()
  }
}
