pub(crate) mod channel;
pub(crate) mod components;
pub(crate) mod error;
pub(crate) mod event_loop;
pub(crate) mod executor;
pub(crate) mod program;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use flow_component::{Component, ComponentError, RuntimeCallback};
use futures::{FutureExt, TryFutureExt};
use parking_lot::Mutex;
use tracing::{info_span, Span};
use tracing_futures::Instrument;
use wick_interface_types::ComponentSignature;
use wick_packet::{Entity, Invocation, PacketStream, RuntimeConfig};

use self::channel::InterpreterDispatchChannel;
use self::components::HandlerMap;
use self::error::Error;
use self::event_loop::EventLoop;
use self::program::Program;
use crate::graph::types::*;
use crate::interpreter::channel::InterpreterChannel;
use crate::interpreter::components::component::ComponentComponent;
use crate::interpreter::components::null::NullComponent;
use crate::interpreter::components::self_component::SelfComponent;
use crate::interpreter::executor::error::ExecutionError;
use crate::{NamespaceHandler, Observer};

#[must_use]
#[derive()]
pub struct Interpreter {
  program: Program,
  event_loop: EventLoop,
  signature: ComponentSignature,
  components: Arc<HandlerMap>,
  self_component: SelfComponent,
  dispatcher: InterpreterDispatchChannel,
  namespace: Option<String>,
  callback: Arc<RuntimeCallback>,
  exposed_ops: HashMap<String, NamespaceHandler>, // A map from op name to the ns of the handler that exposes it.
  span: Span,
}

impl std::fmt::Debug for Interpreter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Interpreter")
      .field("program", &self.program)
      .field("event_loop", &self.event_loop)
      .field("signature", &self.signature)
      .field("components", &self.components)
      .field("dispatcher", &self.dispatcher)
      .finish()
  }
}

impl Interpreter {
  pub fn new(
    network: Network,
    namespace: Option<String>,
    config: Option<RuntimeConfig>,
    components: Option<HandlerMap>,
    callback: Arc<RuntimeCallback>,
    parent_span: &Span,
  ) -> Result<Self, Error> {
    let span = info_span!(parent: parent_span, "interpreter");

    let _guard = span.enter();
    let mut handlers = components.unwrap_or_default();
    debug!(handlers = ?handlers.keys(), "initializing interpreter");
    let mut exposed_ops = HashMap::new();

    let exposed = handlers.inner().values().filter(|h| h.is_exposed()).collect::<Vec<_>>();
    if exposed.len() > 1 {
      return Err(Error::ExposedLimit(
        exposed.iter().map(|h| h.namespace().to_owned()).collect(),
      ));
    }
    let signature = exposed.get(0).copied().map(|handler| {
      for op in &handler.component.signature().operations {
        trace!(operation = op.name, "interpreter:exposing operation");
        exposed_ops.insert(op.name.clone(), handler.clone());
      }
      handler.component.signature().clone()
    });

    handlers.add(NamespaceHandler::new(NullComponent::ID, Box::new(NullComponent::new())))?;

    // Add the component:: component
    let component_component = ComponentComponent::new(&handlers);
    handlers.add(NamespaceHandler::new(
      ComponentComponent::ID,
      Box::new(component_component),
    ))?;

    handlers.add_core(&network)?;

    let mut signatures = handlers.component_signatures();
    program::generate_self_signature(&network, &mut signatures).map_err(Error::EarlyError)?;
    let program = Program::new(network, signatures)?;

    program.validate()?;

    let channel = InterpreterChannel::new();
    let dispatcher = channel.dispatcher(Some(span.clone()));

    // Make the self:: component
    let components = Arc::new(handlers);
    let self_component = SelfComponent::new(components.clone(), program.state(), config, &dispatcher);

    // If we expose a component, expose its signature as our own.
    // Otherwise expose our self signature.
    let signature = signature.unwrap_or_else(|| self_component.signature().clone());

    debug!(?signature, "signature");

    let event_loop = EventLoop::new(channel, &span);
    let mut handled_opts = program.operations().iter().map(|s| s.name()).collect::<Vec<_>>();
    handled_opts.extend(exposed_ops.keys().map(|s: &String| s.as_str()));

    debug!(
      operations = ?handled_opts,
      components = ?components.inner().keys().cloned().collect::<Vec<_>>(),
      "interpreter:scope"
    );
    drop(_guard);

    Ok(Self {
      program,
      dispatcher,
      signature,
      components,
      self_component,
      event_loop,
      namespace,
      exposed_ops,
      callback,
      span,
    })
  }

  fn get_callback(&self) -> Arc<RuntimeCallback> {
    let outside_callback = self.callback.clone();
    let internal_components = self.components.clone();
    let self_component = self.self_component.clone();

    let cb_container = Arc::new(Mutex::new(None));

    let inner_cb = cb_container.clone();
    let local_first_callback: Arc<RuntimeCallback> = Arc::new(move |compref, op, stream, inherent, config, span| {
      let internal_components = internal_components.clone();
      let inner_cb = inner_cb.clone();
      let outside_callback = outside_callback.clone();
      let self_component = self_component.clone();
      let span = span.clone();
      Box::pin(async move {
        span.in_scope(|| trace!(op, %compref, "invoke:component reference"));
        if compref.get_target_id() == SelfComponent::ID {
          span.in_scope(|| trace!(op, %compref, "handling component invocation for self"));
          let cb = inner_cb.lock().clone().unwrap();
          let invocation = compref.to_invocation(&op, stream, inherent, &span);
          self_component.handle(invocation, config, cb).await
        } else if let Some(handler) = internal_components.get(compref.get_target_id()) {
          span.in_scope(|| trace!(op, %compref, "handling component invocation internal to this interpreter"));
          let cb = inner_cb.lock().clone().unwrap();
          let invocation = compref.to_invocation(&op, stream, inherent, &span);
          handler.component().handle(invocation, config, cb).await
        } else {
          outside_callback(compref, op, stream, inherent, config, &span).await
        }
      })
    });
    cb_container.lock().replace(local_first_callback.clone());
    local_first_callback
  }

  pub async fn invoke(&self, invocation: Invocation, config: Option<RuntimeConfig>) -> Result<PacketStream, Error> {
    let cb = self.get_callback();
    let stream = self
      .handle(invocation, config, cb)
      .await
      .map_err(ExecutionError::ComponentError)?;

    Ok(stream)
  }

  pub async fn start(
    &mut self,
    options: Option<InterpreterOptions>,
    observer: Option<Box<dyn Observer + Send + Sync>>,
  ) {
    self.event_loop.start(options.unwrap_or_default(), observer).await;
  }

  pub async fn stop(&self) -> Result<(), Error> {
    let shutdown = self.event_loop.shutdown().await;
    if let Err(error) = &shutdown {
      self.span.in_scope(|| error!(%error,"error shutting down event loop"));
    };
    for (ns, components) in self.components.inner() {
      self
        .span
        .in_scope(|| debug!(namespace = %ns, "shutting down component"));
      if let Err(error) = components
        .component
        .shutdown()
        .await
        .map_err(|e| Error::ComponentShutdown(e.to_string()))
      {
        self.span.in_scope(|| warn!(%error,"error during shutdown"));
      };
    }

    shutdown
  }

  pub fn render_dotviz(&self, op: &str) -> Result<String, Error> {
    self.program.dotviz(op)
  }
}

impl Component for Interpreter {
  fn handle(
    &self,
    mut invocation: Invocation,
    config: Option<RuntimeConfig>,
    cb: Arc<RuntimeCallback>,
  ) -> flow_component::BoxFuture<Result<PacketStream, ComponentError>> {
    let known_targets = || {
      let mut hosted: Vec<_> = self.components.inner().keys().cloned().collect();
      if let Some(ns) = &self.namespace {
        hosted.push(ns.clone());
      }
      hosted
    };
    let span = invocation.span.clone();

    span.in_scope(|| trace!(target=%invocation.target_url(),tx_id=%invocation.tx_id,id=%invocation.id, "invoking"));
    let from_exposed = self.exposed_ops.get(invocation.target.operation_id());

    Box::pin(async move {
      let stream = match &invocation.target {
        Entity::Operation(ns, _) => {
          if ns == SelfComponent::ID || ns == Entity::LOCAL || Some(ns) == self.namespace.as_ref() {
            if let Some(handler) = from_exposed {
              let new_target = Entity::operation(handler.namespace(), invocation.target.operation_id());
              span.in_scope(|| trace!(origin=%invocation.origin,original_target=%invocation.target, %new_target, "invoke::exposed::operation"));
              invocation.target = new_target;
              return handler.component.handle(invocation, config, cb).instrument(span).await;
            }
            span
              .in_scope(|| trace!(origin=%invocation.origin,target=%invocation.target, "invoke::composite::operation"));
            self
              .self_component
              .handle(invocation, config, self.get_callback())
              .await?
          } else if let Some(handler) = self.components.get(ns) {
            span.in_scope(|| trace!(origin=%invocation.origin,target=%invocation.target, "invoke::handler::operation"));
            handler
              .component
              .handle(invocation, config, cb)
              .instrument(span)
              .await?
          } else {
            return Err(ComponentError::new(Error::TargetNotFound(
              invocation.target.clone(),
              known_targets(),
            )));
          }
        }
        _ => return Err(ComponentError::new(Error::InvalidEntity(invocation.target))),
      };

      Ok::<_, ComponentError>(stream)
    })
  }

  fn signature(&self) -> &ComponentSignature {
    &self.signature
  }

  fn shutdown(&self) -> flow_component::BoxFuture<Result<(), ComponentError>> {
    self.stop().map_err(ComponentError::new).boxed()
  }
}

#[derive(Debug, Clone)]
#[allow(missing_copy_implementations)]
pub struct InterpreterOptions {
  /// Timeout after which a component that has received no output is considered dead.
  pub output_timeout: Duration,
}

impl Default for InterpreterOptions {
  fn default() -> Self {
    Self {
      output_timeout: Duration::from_secs(500),
    }
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;

  fn sync_send<T>()
  where
    T: Sync + Send,
  {
  }

  #[test]
  fn test_sync_send() -> Result<()> {
    sync_send::<Interpreter>();
    Ok(())
  }
}
