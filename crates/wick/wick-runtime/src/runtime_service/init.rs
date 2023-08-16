use std::path::Path;

use flow_graph_interpreter::error::InterpreterError;
use flow_graph_interpreter::{HandlerMap, Interpreter};
use wick_config::config::ComponentImplementation;

use super::utils::{assert_constraints, instantiate_import};
use super::{generate_provides_handlers, ChildInit, ComponentRegistry};
use crate::components::validation::expect_signature_match;
use crate::components::{init_impl, make_link_callback};
use crate::dev::prelude::*;
use crate::runtime::{RuntimeConstraint, RuntimeInit};

fn init_err(source: Option<&Path>) -> impl FnOnce(InterpreterError) -> EngineError + '_ {
  move |e| EngineError::InterpreterInit(source.map(Into::into), Box::new(e))
}
#[derive(Debug)]
pub(crate) struct ServiceInit {
  rng: Random,
  pub(crate) id: Uuid,
  pub(crate) manifest: ComponentConfiguration,
  pub(crate) allow_latest: bool,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) namespace: Option<String>,
  pub(crate) constraints: Vec<RuntimeConstraint>,
  pub(crate) initial_components: ComponentRegistry,
  pub(crate) span: Span,
}

impl ServiceInit {
  pub(crate) fn new(seed: Seed, config: RuntimeInit) -> Self {
    let rng = Random::from_seed(seed);
    Self {
      id: rng.uuid(),
      rng,
      manifest: config.manifest,
      allow_latest: config.allow_latest,
      allowed_insecure: config.allowed_insecure,
      namespace: config.namespace,
      constraints: config.constraints,
      initial_components: config.initial_components,
      span: config.span,
    }
  }

  pub(crate) fn new_with_id(id: Uuid, seed: Seed, config: RuntimeInit) -> Self {
    let rng = Random::from_seed(seed);
    Self {
      id,
      rng,
      manifest: config.manifest,
      allow_latest: config.allow_latest,
      allowed_insecure: config.allowed_insecure,
      namespace: config.namespace,
      constraints: config.constraints,
      initial_components: config.initial_components,
      span: config.span,
    }
  }

  pub(super) fn child_init(&self, root_config: Option<RuntimeConfig>, provided: Option<HandlerMap>) -> ChildInit {
    ChildInit {
      rng_seed: self.rng.seed(),
      runtime_id: self.id,
      root_config,
      allow_latest: self.allow_latest,
      allowed_insecure: self.allowed_insecure.clone(),
      provided,
      span: self.span.clone(),
    }
  }

  pub(super) fn seed(&self) -> Seed {
    self.rng.seed()
  }

  pub(super) fn namespace(&self) -> String {
    self.namespace.clone().unwrap_or_else(|| self.id.to_string())
  }

  pub(super) async fn instantiate_main(&self) -> Result<(Option<&[String]>, HandlerMap), EngineError> {
    let mut components = HandlerMap::default();
    let ns = self.namespace.clone().unwrap_or_else(|| self.id.to_string());
    let extends = if let ComponentImplementation::Composite(config) = self.manifest.component() {
      // Initialize a starting set of components for the composite component.
      for factory in self.initial_components.inner() {
        components
          .add(factory(self.seed())?)
          .map_err(init_err(self.manifest.source()))?;
      }
      for id in config.extends() {
        if !self.manifest.import().iter().any(|i| i.id() == id) {
          return Err(EngineError::RuntimeInit(
            self.manifest.source().map(Into::into),
            format!("Inherited component '{}' not found", id),
          ));
        }
      }

      Some(config.extends())
    } else {
      // Instantiate non-composite component as an exposed, standalone component.
      let child_init = self.child_init(self.manifest.root_config().cloned(), None);

      self
        .span
        .in_scope(|| debug!(%ns,options=?child_init,"instantiating component"));

      let component = init_impl(&self.manifest, ns.clone(), child_init, None, Default::default()).await?;
      component.expose();

      expect_signature_match(
        self.manifest.source(),
        component.component().signature(),
        self.manifest.source(),
        &self.manifest.signature()?,
      )?;

      components.add(component).map_err(init_err(self.manifest.source()))?;

      None
    };
    Ok((extends, components))
  }

  pub(super) async fn instantiate_imports(
    &self,
    extends: Option<&[String]>,
    mut components: HandlerMap,
  ) -> Result<HandlerMap, EngineError> {
    for binding in self.manifest.import() {
      let provided = generate_provides_handlers(binding.provide(), &components)?;
      let component_init = self.child_init(binding.config().cloned(), Some(provided));
      if let Some(component) = instantiate_import(binding, component_init, self.manifest.resolver()).await? {
        if let Some(extends) = extends {
          if extends.iter().any(|n| n == component.namespace()) {
            self.span.in_scope(|| {
              component.expose();
              debug!(component = component.namespace(), "extending imported component");
            });
          }
        }
        components.add(component).map_err(init_err(self.manifest.source()))?;
      }
    }
    assert_constraints(&self.constraints, &components)?;
    Ok(components)
  }

  pub(super) async fn init_interpreter(&mut self, components: HandlerMap) -> Result<Interpreter, EngineError> {
    let graph = self.span.in_scope(|| {
      debug!("generating graph");
      flow_graph_interpreter::graph::from_def(&mut self.manifest, &components)
        .map_err(|e| EngineError::Graph(self.manifest.source().map(Into::into), Box::new(e)))
    })?;

    let mut interpreter = flow_graph_interpreter::Interpreter::new(
      graph,
      Some(self.namespace()),
      Some(components),
      make_link_callback(self.id),
      &self.span,
    )
    .map_err(init_err(self.manifest.source()))?;
    interpreter.start(None, None).await;
    Ok(interpreter)
  }
}
