pub(crate) mod channel;
pub(crate) mod components;
pub(crate) mod error;
pub(crate) mod event_loop;
pub(crate) mod executor;
pub(crate) mod program;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use flow_component::{Component, RuntimeCallback};
use parking_lot::Mutex;
use seeded_random::{Random, Seed};
use tracing_futures::Instrument;
use wick_interface_types::ComponentSignature;
use wick_packet::{Entity, Invocation, OperationConfig, PacketStream};

use self::channel::InterpreterDispatchChannel;
use self::components::HandlerMap;
use self::error::Error;
use self::event_loop::EventLoop;
use self::executor::SchematicExecutor;
use self::program::Program;
use crate::constants::*;
use crate::graph::types::*;
use crate::interpreter::channel::InterpreterChannel;
use crate::interpreter::components::component_component::ComponentComponent;
use crate::interpreter::components::schematic_component::SchematicComponent;
use crate::interpreter::executor::error::ExecutionError;
use crate::{NamespaceHandler, Observer, SharedHandler};

#[must_use]
#[derive()]
pub struct Interpreter {
  rng: Random,
  program: Program,
  event_loop: EventLoop,
  signature: ComponentSignature,
  components: Arc<HandlerMap>,
  self_component: Arc<SchematicComponent>,
  dispatcher: InterpreterDispatchChannel,
  namespace: Option<String>,
  callback: Arc<RuntimeCallback>,
  exposed_ops: HashMap<String, SharedHandler>, // A map from op name to the ns of the handler that exposes it.
}

impl std::fmt::Debug for Interpreter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Interpreter")
      .field("program", &self.program)
      .field("event_loop", &self.event_loop)
      .field("signature", &self.signature)
      .field("collections", &self.components)
      .field("dispatcher", &self.dispatcher)
      .finish()
  }
}

impl Interpreter {
  #[instrument(name="interpreter-init", skip_all, fields(namespace = %namespace.as_ref().map_or("n/a", String::as_str)))]
  pub fn new(
    seed: Option<Seed>,
    network: Network,
    namespace: Option<String>,
    components: Option<HandlerMap>,
    callback: Arc<RuntimeCallback>,
  ) -> Result<Self, Error> {
    debug!("init");
    let rng = seed.map_or_else(Random::new, Random::from_seed);
    let mut handlers = components.unwrap_or_default();
    debug!(handlers = ?handlers.keys(), "initializing interpreter");
    let mut exposed_ops = HashMap::new();

    for handler in handlers.inner().values() {
      if handler.is_exposed() {
        for op in &handler.component.list().operations {
          trace!(operation = op.name, "interpreter:exposing operation");
          exposed_ops.insert(op.name.clone(), handler.component.clone());
        }
      }
    }
    handlers.add_core(&network)?;

    // Add the component:: component
    let component_component = ComponentComponent::new(&handlers);
    handlers.add(NamespaceHandler::new(NS_COMPONENTS, Box::new(component_component)))?;

    let mut signatures = handlers.component_signatures();
    program::generate_self_signature(&network, &mut signatures).map_err(Error::EarlyError)?;
    let program = Program::new(network, signatures)?;

    program.validate()?;

    let channel = InterpreterChannel::new();
    let dispatcher = channel.dispatcher();

    // Make the self:: component
    let components = Arc::new(handlers);
    let self_component = SchematicComponent::new(components.clone(), program.state(), &dispatcher, rng.seed());
    let self_signature = self_component.list().clone();

    debug!(?self_signature, "signature");

    let event_loop = EventLoop::new(channel);
    let mut handled_opts = program.operations().iter().map(|s| s.name()).collect::<Vec<_>>();
    handled_opts.extend(exposed_ops.keys().map(|s: &String| s.as_str()));
    debug!(
      operations = ?handled_opts,
      "operations handled by this interpreter"
    );

    Ok(Self {
      rng,
      program,
      dispatcher,
      signature: self_signature,
      components,
      self_component,
      event_loop,
      namespace,
      exposed_ops,
      callback,
    })
  }

  fn get_callback(&self) -> Arc<RuntimeCallback> {
    let outside_callback = self.callback.clone();
    let internal_components = self.components.clone();
    let self_component = self.self_component.clone();

    let cb_container = Arc::new(Mutex::new(None));

    let inner_cb = cb_container.clone();
    let local_first_callback: Arc<RuntimeCallback> = Arc::new(move |compref, op, stream, inherent, config| {
      let internal_components = internal_components.clone();
      let inner_cb = inner_cb.clone();
      let outside_callback = outside_callback.clone();
      let self_component = self_component.clone();
      Box::pin(async move {
        trace!(op, %compref, "invoke:component reference");
        if compref.get_target_id() == NS_SELF {
          trace!(op, %compref, "handling component invocation for self");
          let cb = inner_cb.lock().clone().unwrap();
          let invocation = compref.to_invocation(&op, inherent);
          self_component.handle(invocation, stream, config, cb).await
        } else if let Some(handler) = internal_components.get(compref.get_target_id()) {
          trace!(op, %compref, "handling component invocation internal to this interpreter");
          let cb = inner_cb.lock().clone().unwrap();
          let invocation = compref.to_invocation(&op, inherent);
          handler.component().handle(invocation, stream, config, cb).await
        } else {
          outside_callback(compref, op, stream, inherent, config).await
        }
      })
    });
    cb_container.lock().replace(local_first_callback.clone());
    local_first_callback
  }

  async fn invoke_operation(&self, invocation: Invocation, stream: PacketStream) -> Result<PacketStream, Error> {
    let dispatcher = self.dispatcher.clone();
    let name = invocation.target.operation_id().to_owned();
    let op = self
      .program
      .operations()
      .iter()
      .find(|s| s.name() == name)
      .ok_or_else(|| {
        Error::OpNotFound(
          invocation.target.clone(),
          self.program.operations().iter().map(|s| s.name().to_owned()).collect(),
        )
      })?;

    let executor = SchematicExecutor::new(op.clone(), dispatcher.clone());
    Ok(
      executor
        .invoke(
          invocation,
          stream,
          self.rng.seed(),
          self.components.clone(),
          self.self_component.clone(),
          self.get_callback(),
        )
        .instrument(tracing::span::Span::current())
        .await?,
    )
  }

  pub async fn invoke(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    config: Option<OperationConfig>,
  ) -> Result<PacketStream, Error> {
    let known_targets = || {
      let mut hosted: Vec<_> = self.components.inner().keys().cloned().collect();
      if let Some(ns) = &self.namespace {
        hosted.push(ns.clone());
      }
      hosted
    };
    let span = trace_span!("invoke");
    let cb = self.get_callback();
    trace!(?invocation, "invoking");
    let stream = match &invocation.target {
      Entity::Operation(ns, name) => {
        if ns == NS_SELF || ns == Entity::LOCAL || Some(ns) == self.namespace.as_ref() {
          if let Some(component) = self.exposed_ops.get(name) {
            trace!(entity=%invocation.target, "invoke::exposed::operation");
            return Ok(
              component
                .handle(invocation, stream, config, cb)
                .instrument(span)
                .await
                .map_err(ExecutionError::ComponentError)?,
            );
          }
          trace!(entity=%invocation.target, "invoke::composite::operation");
          self.invoke_operation(invocation, stream).instrument(span).await?
        } else {
          trace!(entity=%invocation.target, "invoke::instance::operation");
          self
            .components
            .get(ns)
            .ok_or_else(|| Error::TargetNotFound(invocation.target.clone(), known_targets()))?
            .component
            .handle(invocation, stream, config, cb)
            .instrument(span)
            .await
            .map_err(ExecutionError::ComponentError)?
        }
      }
      _ => return Err(Error::TargetNotFound(invocation.target, known_targets())),
    };

    Ok(stream)
  }

  pub fn get_export_signature(&self) -> &ComponentSignature {
    &self.signature
  }

  pub async fn start(
    &mut self,
    options: Option<InterpreterOptions>,
    observer: Option<Box<dyn Observer + Send + Sync>>,
  ) {
    self.event_loop.start(options.unwrap_or_default(), observer).await;
  }

  #[instrument(skip(self))]
  pub async fn shutdown(&self) -> Result<(), Error> {
    let shutdown = self.event_loop.shutdown().await;
    if let Err(error) = &shutdown {
      error!(%error,"error shutting down event loop");
    };
    for (ns, components) in self.components.inner() {
      debug!(namespace = %ns, "shutting down collection");
      if let Err(error) = components
        .component
        .shutdown()
        .await
        .map_err(|e| Error::ComponentShutdown(e.to_string()))
      {
        warn!(%error,"error during shutdown");
      };
    }

    shutdown
  }
}

#[derive(Debug, Clone)]
#[allow(missing_copy_implementations)]
pub struct InterpreterOptions {
  /// Stop the interpreter and return an error on any hung transactions.
  pub error_on_hung: bool,
  /// Stop the interpreter and return an error if any messages come after a transaction has completed.
  pub error_on_missing: bool,
  /// Timeout after which a component that has received no output is considered dead.
  pub output_timeout: Duration,
  /// Timeout after which a transaction that has had no events is considered hung.
  pub hung_tx_timeout: Duration,
}

impl Default for InterpreterOptions {
  fn default() -> Self {
    Self {
      error_on_hung: false,
      error_on_missing: false,
      output_timeout: Duration::from_secs(5),
      hung_tx_timeout: Duration::from_millis(500),
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

  #[test_logger::test]
  fn test_sync_send() -> Result<()> {
    sync_send::<Interpreter>();
    Ok(())
  }
}
